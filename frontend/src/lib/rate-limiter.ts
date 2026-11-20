/**
 * Shared rate limiter for Next.js API routes.
 * Provides per-IP rate limiting with automatic window expiry to prevent memory leaks.
 */

interface RateLimitEntry {
  count: number;
  windowStart: number;
}

/**
 * Hard cap on tracked keys, as a backstop against a single-window flood of
 * unique keys (e.g. a spoofed-IP flood) outpacing the expiry sweep below.
 */
const MAX_TRACKED_KEYS = 50_000;

/** A rate limiter function that also exposes its current tracked-key count. */
export interface RateLimiter {
  (ip: string): boolean;
  /** Number of keys currently tracked (for tests/ops monitoring). */
  readonly size: number;
}

/**
 * Creates a rate limiter function with specified limits and window duration.
 * The limiter automatically cleans up expired entries to prevent memory leaks.
 *
 * @param maxRequests Maximum requests allowed per window
 * @param windowMs Duration of the rate limit window in milliseconds
 */
export function createRateLimiter(
  maxRequests: number,
  windowMs: number,
): RateLimiter {
  const rateLimitMap = new Map<string, RateLimitEntry>();

  // Map iteration order mirrors insertion order, and every entry is
  // re-inserted (delete + set) whenever its window resets — see below — so
  // the oldest `windowStart` is always first. Sweeping from the front lets
  // us stop as soon as we hit a still-fresh entry instead of scanning the
  // whole map on every call.
  function evictExpired(now: number): void {
    for (const [key, entry] of rateLimitMap) {
      if (now - entry.windowStart > windowMs) {
        rateLimitMap.delete(key);
      } else {
        break;
      }
    }
  }

  function enforceCap(): void {
    while (rateLimitMap.size > MAX_TRACKED_KEYS) {
      const oldestKey = rateLimitMap.keys().next().value;
      if (oldestKey === undefined) break;
      rateLimitMap.delete(oldestKey);
    }
  }

  const limiter = ((ip: string): boolean => {
    const now = Date.now();
    evictExpired(now);

    const entry = rateLimitMap.get(ip);
    let limited: boolean;
    if (!entry || now - entry.windowStart > windowMs) {
      // Delete before re-inserting so the key moves to the end of the map's
      // iteration order, keeping evictExpired's oldest-first assumption true.
      rateLimitMap.delete(ip);
      rateLimitMap.set(ip, { count: 1, windowStart: now });
      limited = false;
    } else {
      entry.count += 1;
      limited = entry.count > maxRequests;
    }

    // Enforced after every call so size never exceeds the cap, even within
    // a single still-fresh window where evictExpired can't remove anything.
    enforceCap();
    return limited;
  }) as RateLimiter;

  Object.defineProperty(limiter, "size", {
    get: () => rateLimitMap.size,
  });

  return limiter;
}

/**
 * Extracts the client IP address from request headers.
 * Checks x-forwarded-for (proxy) and x-real-ip (nginx) headers.
 * **Security note**: These headers can be spoofed if the request is not from a
 * trusted proxy. Ensure your reverse proxy (e.g., Cloudflare, AWS ALB) sets these
 * headers and that your application only trusts requests from known proxies.
 *
 * @param headers Request headers
 * @returns Client IP address or "unknown" if not found
 */
export function getClientIp(headers: Headers): string {
  // x-forwarded-for may contain multiple IPs (client, proxy1, proxy2, ...)
  // Take the first IP which should be the client
  const forwarded = headers.get("x-forwarded-for")?.split(",")[0]?.trim();
  if (forwarded) return forwarded;

  const realIp = headers.get("x-real-ip")?.trim();
  if (realIp) return realIp;

  return "unknown";
}
