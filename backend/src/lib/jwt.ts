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

/**
 * Custom error class for invalid tokens.
 * Allows callers to distinguish token errors from other exceptions.
 */
export class InvalidTokenError extends Error {
  constructor(message: string = "Invalid token") {
    super(message);
    this.name = "InvalidTokenError";
  }
}

export interface JwtPayload {
  sub?: string;
  iat?: number;
  exp: number;
  [claim: string]: unknown;
}

export function createJwt(payload: Record<string, unknown>): string {
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

/**
 * Decode and verify a JWT token.
 * 
 * Security-critical flow:
 * 1. Check segment count (must be 3)
 * 2. Verify HMAC signature over raw segments BEFORE parsing
 * 3. Parse header and validate alg/typ
 * 4. Parse payload
 * 5. Check expiry
 * 
 * All failures throw InvalidTokenError with a safe, generic message.
 * No SyntaxError or internal parser details ever escape.
 */
export function decodeJwt(token: string): JwtPayload {
  try {
    const parts = token.split(".");
    if (parts.length !== 3) {
      throw new InvalidTokenError("Invalid token format");
    }

    const [headerB64, bodyB64, sig] = parts;

    // CRITICAL: Verify signature BEFORE parsing anything
    const expected = crypto
      .createHmac("sha256", JWT_SECRET)
      .update(`${headerB64}.${bodyB64}`)
      .digest("base64url");

    if (!safeCompareSignatures(sig, expected)) {
      throw new InvalidTokenError("Invalid token signature");
    }

    // Signature verified; now safe to parse header
    let header: { alg?: unknown; typ?: unknown };
    try {
      header = JSON.parse(Buffer.from(headerB64, "base64url").toString());
    } catch {
      throw new InvalidTokenError("Invalid token header");
    }

    if (header.alg !== "HS256" || header.typ !== "JWT") {
      throw new InvalidTokenError("Invalid token header");
    }

    // Parse payload
    let payload: Record<string, unknown>;
    try {
      payload = JSON.parse(Buffer.from(bodyB64, "base64url").toString());
    } catch {
      throw new InvalidTokenError("Invalid token payload");
    }

    // exp must be present and a finite number
    if (typeof payload.exp !== "number" || !Number.isFinite(payload.exp)) {
      throw new InvalidTokenError("Token missing or invalid exp claim");
    }

    if (payload.exp < Math.floor(Date.now() / 1000)) {
      throw new InvalidTokenError("Token expired");
    }

    return payload as JwtPayload;
  } catch (error) {
    // Convert any unexpected errors to InvalidTokenError
    if (error instanceof InvalidTokenError) {
      throw error;
    }
    throw new InvalidTokenError("Invalid token");
  }
}

export { TOKEN_EXPIRY };
