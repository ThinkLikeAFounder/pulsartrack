import type { Event } from "@sentry/nextjs";

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

export function scrubEvent<T extends Event>(event: T): T {
  return scrub(event) as T;
}
