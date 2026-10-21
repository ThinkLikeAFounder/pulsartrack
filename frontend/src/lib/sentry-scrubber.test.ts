/**
 * Frontend Sentry Scrubber Tests
 *
 * Verifies that the frontend's thin re-export of @pulsartrack/sentry-scrubber
 * correctly scrubs sensitive values and identifies sensitive keys.
 */

import { describe, it, expect } from "vitest";

describe("Frontend Sentry Scrubber", () => {
  it("scrubString redacts Bearer tokens and does not leak the raw value", async () => {
    const { scrubString } = await import("./sentry-scrubber");
    const result = scrubString("test Bearer token123");
    expect(result).not.toContain("token123");
    expect(result).toContain("[Filtered]");
  });

  it("isSensitiveKey returns true for sensitive key names", async () => {
    const { isSensitiveKey } = await import("./sentry-scrubber");
    expect(isSensitiveKey("token")).toBe(true);
    expect(isSensitiveKey("password")).toBe(true);
    expect(isSensitiveKey("secret")).toBe(true);
  });

  it("isSensitiveKey returns false for non-sensitive key names", async () => {
    const { isSensitiveKey } = await import("./sentry-scrubber");
    expect(isSensitiveKey("userId")).toBe(false);
    expect(isSensitiveKey("keyboard")).toBe(false);
  });
});
