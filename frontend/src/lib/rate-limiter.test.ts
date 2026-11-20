import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { createRateLimiter, getClientIp } from "./rate-limiter";

describe("createRateLimiter", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("allows up to maxRequests within a window, then blocks", () => {
    const limiter = createRateLimiter(3, 60_000);

    expect(limiter("1.1.1.1")).toBe(false); // 1st
    expect(limiter("1.1.1.1")).toBe(false); // 2nd
    expect(limiter("1.1.1.1")).toBe(false); // 3rd
    expect(limiter("1.1.1.1")).toBe(true); // 4th — over the limit
  });

  it("resets a key's count once its window has elapsed", () => {
    const limiter = createRateLimiter(1, 60_000);

    expect(limiter("1.1.1.1")).toBe(false);
    expect(limiter("1.1.1.1")).toBe(true);

    vi.advanceTimersByTime(60_001);

    expect(limiter("1.1.1.1")).toBe(false);
  });

  it("tracks different keys independently", () => {
    const limiter = createRateLimiter(1, 60_000);

    expect(limiter("1.1.1.1")).toBe(false);
    expect(limiter("2.2.2.2")).toBe(false);
    expect(limiter("1.1.1.1")).toBe(true);
    expect(limiter("2.2.2.2")).toBe(true);
  });

  it("evicts entries for keys that stop sending requests once their window expires", () => {
    const limiter = createRateLimiter(5, 60_000);

    for (let i = 0; i < 100; i++) {
      limiter(`10.0.0.${i}`);
    }
    expect(limiter.size).toBe(100);

    // None of those 100 keys ever come back. Advance past their window and
    // make one more call (from a new key) — the stale entries must be swept
    // even though nobody asked about them specifically.
    vi.advanceTimersByTime(60_001);
    limiter("1.2.3.4");

    expect(limiter.size).toBe(1);
  });

  it("does not grow unboundedly under a sustained spoofed-IP flood", () => {
    const limiter = createRateLimiter(60, 60_000);
    const start = Date.now();

    // Simulate an attacker rotating a spoofed IP on every request, well
    // past several rate-limit windows.
    for (let i = 0; i < 500; i++) {
      vi.setSystemTime(start + i * 1_000); // one unique spoofed "IP" per second
      limiter(`spoofed-ip-${i}`);
    }

    // Only requests from within the trailing window can still be live;
    // everything older must have been evicted along the way, not just
    // accumulated for the whole 500-request run.
    expect(limiter.size).toBeLessThanOrEqual(61);
  });

  it("caps tracked keys even within a single window via the hard size backstop", () => {
    const limiter = createRateLimiter(5, 60_000);

    // All within the same instant/window, so nothing is eligible for the
    // normal expiry sweep yet — only the hard cap can bound growth here.
    for (let i = 0; i < 60_100; i++) {
      limiter(`flood-${i}`);
    }

    expect(limiter.size).toBeLessThanOrEqual(50_000);
  });
});

describe("getClientIp", () => {
  it("prefers the first address in x-forwarded-for", () => {
    const headers = new Headers({ "x-forwarded-for": "1.1.1.1, 2.2.2.2, 3.3.3.3" });
    expect(getClientIp(headers)).toBe("1.1.1.1");
  });

  it("falls back to x-real-ip when x-forwarded-for is absent", () => {
    const headers = new Headers({ "x-real-ip": "9.9.9.9" });
    expect(getClientIp(headers)).toBe("9.9.9.9");
  });

  it("returns 'unknown' when neither header is present", () => {
    const headers = new Headers();
    expect(getClientIp(headers)).toBe("unknown");
  });
});
