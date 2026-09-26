import { describe, expect, it } from "vitest";
import fs from "fs";
import path from "path";
import { isSensitiveKey, scrubEvent, scrubString } from "./sentry-scrubber";

describe("Frontend Sentry Scrubber", () => {
  it("should recognize sensitive keys via pattern matching", () => {
    expect(isSensitiveKey("authToken")).toBe(true);
    expect(isSensitiveKey("jwtSecret")).toBe(true);
    expect(isSensitiveKey("secret_key")).toBe(true);
    expect(isSensitiveKey("private_key")).toBe(true);
    expect(isSensitiveKey("seed")).toBe(true);
    expect(isSensitiveKey("userMnemonic")).toBe(true);
    expect(isSensitiveKey("password")).toBe(true);
    expect(isSensitiveKey("x-api-key")).toBe(true);
    expect(isSensitiveKey("bearer")).toBe(true);
    expect(isSensitiveKey("session_id")).toBe(true);
    expect(isSensitiveKey("username")).toBe(true);
  });

  it("should scrub Stellar secret seeds from string values", () => {
    const seed = "SD123456789012345678901234567890123456789012345678901234";
    expect(seed).toHaveLength(56);
    const input = `Error failed with seed ${seed} on network`;
    const result = scrubString(input);
    expect(result).toBe("Error failed with seed [Filtered] on network");
  });

  it("should scrub JWT tokens from string values", () => {
    const jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    const input = `User token is ${jwt}`;
    const result = scrubString(input);
    expect(result).toBe("User token is [Filtered]");
  });

  it("should scrub Bearer credentials from string values", () => {
    const input = "Authorization header: Bearer mySecretToken123";
    const result = scrubString(input);
    expect(result).toBe("Authorization header: Bearer [Filtered]");
  });

  it("should scrub sensitive query parameters from URLs and strings", () => {
    const url = "https://api.pulsartrack.com/v1/data?token=secret123&key=abc456&public=true";
    const result = scrubString(url);
    expect(result).toBe("https://api.pulsartrack.com/v1/data?token=[Filtered]&key=[Filtered]&public=true");
  });

  it("should scrub nested Sentry events containing seeds, tokens, and sensitive keys", () => {
    const seed = "SA111111111111111111111111111111111111111111111111111111";
    const event = {
      exception: {
        values: [
          {
            type: "Error",
            value: `Critical failure using seed ${seed} during transaction`,
          },
        ],
      },
      request: {
        url: "https://api.pulsartrack.com/pay?token=secretTokenValue&amount=10",
      },
      breadcrumbs: [
        {
          message: "Sending header Bearer topSecretToken",
        },
      ],
      extra: {
        jwtSecret: "unscrubbedSecretValue",
        safeValue: 12345,
      },
    };

    const scrubbed = scrubEvent(event);
    expect(scrubbed.exception.values[0].value).toBe("Critical failure using seed [Filtered] during transaction");
    expect(scrubbed.request.url).toBe("https://api.pulsartrack.com/pay?token=[Filtered]&amount=10");
    expect(scrubbed.breadcrumbs[0].message).toBe("Sending header Bearer [Filtered]");
    expect(scrubbed.extra.jwtSecret).toBe("[Filtered]");
    expect(scrubbed.extra.safeValue).toBe(12345);
  });

  it("should keep frontend and backend sentry-scrubber files in sync", () => {
    const frontendPath = path.resolve(__dirname, "./sentry-scrubber.ts");
    const backendPath = path.resolve(__dirname, "../../../backend/src/lib/sentry-scrubber.ts");
    const frontendContent = fs.readFileSync(frontendPath, "utf-8");
    const backendContent = fs.readFileSync(backendPath, "utf-8");
    expect(frontendContent).toBe(backendContent);
  });
});
