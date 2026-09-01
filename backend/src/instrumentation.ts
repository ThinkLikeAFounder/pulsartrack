import "dotenv/config";
import * as Sentry from "@sentry/node";

const SENSITIVE_KEYS = new Set([
  "authorization",
  "cookie",
  "set-cookie",
  "password",
  "token",
  "access_token",
  "api_key",
  "apikey",
  "email",
  "ip_address",
  "phone",
  "refresh_token",
  "session",
  "jwt",
  "secret",
  "username",
]);

function scrub(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(scrub);
  if (!value || typeof value !== "object") return value;

  return Object.fromEntries(
    Object.entries(value).map(([key, item]) => [
      key,
      SENSITIVE_KEYS.has(key.toLowerCase()) ? "[Filtered]" : scrub(item),
    ]),
  );
}

Sentry.init({
  dsn: process.env.SENTRY_DSN,
  enabled: Boolean(process.env.SENTRY_DSN),
  environment: process.env.NODE_ENV,
  sendDefaultPii: false,
  beforeSend(event) {
    return scrub(event) as typeof event;
  },
});
