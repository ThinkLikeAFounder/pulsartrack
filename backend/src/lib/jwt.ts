import crypto from "crypto";
import { logger } from "./logger";

const TOKEN_EXPIRY = 3600;

const envJwtSecret = process.env.JWT_SECRET;
if (!envJwtSecret) {
  if (process.env.NODE_ENV === "production") {
    throw new Error("JWT_SECRET environment variable is required in production");
  }
  logger.warn("JWT_SECRET not set; using random secret (tokens will not survive restarts)");
}

const JWT_SECRET = envJwtSecret ?? crypto.randomBytes(32).toString("hex");

export function createJwt(payload: Record<string, any>): string {
  const header = Buffer.from(
    JSON.stringify({ alg: "HS256", typ: "JWT" }),
  ).toString("base64url");
  const now = Math.floor(Date.now() / 1000);
  const body = Buffer.from(
    JSON.stringify({ ...payload, iat: now, exp: now + TOKEN_EXPIRY }),
  ).toString("base64url");
  const sig = crypto
    .createHmac("sha256", JWT_SECRET)
    .update(`${header}.${body}`)
    .digest("base64url");
  return `${header}.${body}.${sig}`;
}

/**
 * Constant-time comparison of two base64url signatures.
 *
 * `crypto.timingSafeEqual` throws when the buffers differ in length, and the
 * length itself is not secret (it is a fixed-width HMAC digest), so a
 * length mismatch is rejected up front without leaking byte-position timing.
 */
function safeCompareSignatures(actual: string, expected: string): boolean {
  const actualBuf = Buffer.from(actual, "base64url");
  const expectedBuf = Buffer.from(expected, "base64url");
  if (actualBuf.length !== expectedBuf.length) return false;
  return crypto.timingSafeEqual(actualBuf, expectedBuf);
}

export function decodeJwt(token: string): Record<string, any> {
  const parts = token.split(".");
  if (parts.length !== 3) throw new Error("Malformed token");
  const [header, body, sig] = parts;
  const expected = crypto
    .createHmac("sha256", JWT_SECRET)
    .update(`${header}.${body}`)
    .digest("base64url");
  if (!safeCompareSignatures(sig, expected)) {
    throw new Error("Invalid token signature");
  }
  const payload = JSON.parse(Buffer.from(body, "base64url").toString());
  if (payload.exp < Math.floor(Date.now() / 1000)) {
    throw new Error("Token expired");
  }
  return payload;
}

export { TOKEN_EXPIRY };
