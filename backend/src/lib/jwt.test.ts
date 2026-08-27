import { describe, it, expect } from "vitest";
import crypto from "crypto";
import { createJwt, decodeJwt, TOKEN_EXPIRY } from "./jwt";

const JWT_SECRET = process.env.JWT_SECRET as string;

function sign(header: string, body: string, secret = JWT_SECRET): string {
  return crypto
    .createHmac("sha256", secret)
    .update(`${header}.${body}`)
    .digest("base64url");
}

function encode(obj: Record<string, any>): string {
  return Buffer.from(JSON.stringify(obj)).toString("base64url");
}

describe("jwt", () => {
  const address = "GB7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7";

  it("round-trips a valid token", () => {
    const payload = decodeJwt(createJwt({ sub: address }));
    expect(payload.sub).toBe(address);
    expect(payload.exp - payload.iat).toBe(TOKEN_EXPIRY);
  });

  it("rejects a token whose payload was tampered with", () => {
    const [header, , sig] = createJwt({ sub: address }).split(".");
    const now = Math.floor(Date.now() / 1000);
    const forgedBody = encode({
      sub: "GATTACKER000000000000000000000000000000000000000000000",
      iat: now,
      exp: now + TOKEN_EXPIRY,
    });

    expect(() => decodeJwt(`${header}.${forgedBody}.${sig}`)).toThrow(
      "Invalid token signature",
    );
  });

  it("rejects a signature forged with a different secret", () => {
    const [header, body] = createJwt({ sub: address }).split(".");
    const forgedSig = sign(header, body, "not-the-real-secret");

    expect(() => decodeJwt(`${header}.${body}.${forgedSig}`)).toThrow(
      "Invalid token signature",
    );
  });

  it("rejects a signature of the wrong length without throwing from timingSafeEqual", () => {
    const [header, body] = createJwt({ sub: address }).split(".");

    for (const badSig of ["", "abc", sign(header, body) + "extra"]) {
      expect(() => decodeJwt(`${header}.${body}.${badSig}`)).toThrow(
        "Invalid token signature",
      );
    }
  });

  it("rejects a signature that differs only in the final byte", () => {
    const [header, body, sig] = createJwt({ sub: address }).split(".");
    const buf = Buffer.from(sig, "base64url");
    buf[buf.length - 1] ^= 0xff;

    expect(() =>
      decodeJwt(`${header}.${body}.${buf.toString("base64url")}`),
    ).toThrow("Invalid token signature");
  });

  it("rejects a malformed token", () => {
    expect(() => decodeJwt("only.two")).toThrow("Malformed token");
  });

  it("rejects an expired token", () => {
    const header = encode({ alg: "HS256", typ: "JWT" });
    const past = Math.floor(Date.now() / 1000) - TOKEN_EXPIRY * 2;
    const body = encode({ sub: address, iat: past, exp: past + TOKEN_EXPIRY });

    expect(() => decodeJwt(`${header}.${body}.${sign(header, body)}`)).toThrow(
      "Token expired",
    );
  });

});
