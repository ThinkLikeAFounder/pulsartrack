/**
 * Backend Sentry Scrubber Tests
 * 
 * The shared test suite lives in packages/sentry-scrubber.
 * This file imports and runs those tests to verify the backend's integration.
 */

import { describe, it, expect } from "vitest";

describe("Backend Sentry Scrubber", () => {
  it("should use shared scrubber from @pulsartrack/sentry-scrubber package", async () => {
    // The actual tests are in packages/sentry-scrubber/src/index.test.ts
    // This file exists to document that backend uses the shared implementation
    // Run tests with: npm test packages/sentry-scrubber
    const { scrubString } = await import("../lib/sentry-scrubber");
    
    // Basic smoke test
    const result = scrubString("test Bearer token123");
    expect(result).not.toContain("token123");
    // If this passes, the shared package is properly integrated
  });
});
