import { NextResponse } from "next/server";

/**
 * CSP violation report endpoint. Browsers POST a JSON payload here when a
 * Content-Security-Policy violation occurs (via the `report-uri` directive).
 *
 * In production, consider forwarding to Sentry or a dedicated logging
 * service. For now, log to stdout so violations are visible in container
 * logs (`docker compose logs frontend`).
 *
 * @see https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP
 */
export async function POST(request: Request) {
  try {
    const report = await request.json();
    // The report format varies by browser; the useful fields are typically
    // `csp-report` → { "document-uri", "violated-directive", "blocked-uri" }.
    const cspReport = report["csp-report"] ?? report;
    console.warn(
      "[CSP-VIOLATION]",
      JSON.stringify({
        documentUri: cspReport["document-uri"],
        violatedDirective: cspReport["violated-directive"],
        blockedUri: cspReport["blocked-uri"],
        sourceFile: cspReport["source-file"],
        lineNumber: cspReport["line-number"],
      }),
    );
  } catch {
    // Malformed report — nothing we can do.
  }
  return new NextResponse(null, { status: 204 });
}
