import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";
import { buildCsp } from "./lib/security-headers";

/**
 * Generate a cryptographically random nonce (base64) for CSP script-src.
 * Each request gets a unique nonce so inline scripts can be allowed without
 * opening the door to injected scripts.
 */
function generateNonce(): string {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return btoa(String.fromCharCode(...bytes));
}

/**
 * Proxy runs on every request and sets the Content-Security-Policy header
 * with a per-request nonce. This replaces the build-time CSP from
 * next.config.ts, which could not use nonces (evaluated once at build)
 * and defaulted to report-only with no reporting endpoint (#918).
 *
 * The nonce and CSP are forwarded on the **request** headers so that
 * Next.js's server renderer can read them (via `headers()`) and attach
 * the nonce to its own bootstrap/hydration <script> tags. The CSP is
 * also written to the **response** headers so the browser enforces it.
 *
 * CSP_MODE env var controls enforcement:
 *   - "enforce" → Content-Security-Policy (blocks violations)
 *   - anything else or unset → Content-Security-Policy-Report-Only
 *
 * Note: pages that are statically prerendered at build time cannot receive
 * a per-request nonce. Those routes must be converted to dynamic rendering
 * (e.g. `export const dynamic = 'force-dynamic'`) before switching
 * CSP_MODE to "enforce".
 */
export function proxy(request: NextRequest) {
  const nonce = generateNonce();
  const isDev = process.env.NODE_ENV !== "production";
  const enforceCsp = process.env.CSP_MODE === "enforce";
  const reportTo = process.env.NEXT_PUBLIC_CSP_REPORT_URI || "/api/csp-report";

  const csp = buildCsp({
    wsUrl: process.env.NEXT_PUBLIC_WS_URL,
    enforceCsp,
    isDev,
    nonce,
    reportTo,
  });

  const cspHeaderName = enforceCsp
    ? "Content-Security-Policy"
    : "Content-Security-Policy-Report-Only";

  // Forward the nonce and CSP on the request so the Next.js renderer (server
  // components / layouts) can read them via `headers()` and inject the nonce
  // on <script> tags. Without this, Next.js never sees the nonce and its own
  // bootstrap scripts will be blocked when enforcement is enabled.
  const requestHeaders = new Headers(request.headers);
  requestHeaders.set("X-Nonce", nonce);
  requestHeaders.set(cspHeaderName, csp);

  const response = NextResponse.next({
    request: { headers: requestHeaders },
  });

  // Also set security headers on the response so the browser enforces the policy.
  response.headers.set(cspHeaderName, csp);
  response.headers.set("X-Content-Type-Options", "nosniff");
  response.headers.set("X-Frame-Options", "DENY");
  response.headers.set("Referrer-Policy", "strict-origin-when-cross-origin");
  response.headers.set(
    "Permissions-Policy",
    "camera=(), microphone=(), geolocation=()",
  );
  response.headers.set(
    "Strict-Transport-Security",
    "max-age=63072000; includeSubDomains",
  );

  return response;
}

export const config = {
  matcher: [
    /*
     * Match all request paths except:
     * - _next/static (static files)
     * - _next/image (image optimization)
     * - favicon.ico (browser icon)
     * - public files
     */
    "/((?!_next/static|_next/image|favicon.ico|.*\\.(?:svg|png|jpg|jpeg|gif|webp)$).*)",
  ],
};
