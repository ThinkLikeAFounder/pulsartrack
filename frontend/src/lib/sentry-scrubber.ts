const SENSITIVE_KEY_PATTERN =
  /(?:token|secret|key|seed|mnemonic|password|bearer|auth|credential|private|cookie|session|jwt|email|phone|ip_address)/i;

const STELLAR_SEED_PATTERN = /\bS[A-Z2-7]{55}\b/g;
const JWT_PATTERN = /\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b/g;
const BEARER_PATTERN = /(bearer\s+)[^\s"']+/gi;
const SENSITIVE_QUERY_PARAM_PATTERN =
  /([?&](?:token|key|secret|api_key|apikey|password|seed|mnemonic|bearer|auth|access_token|refresh_token|private_key|x-api-key)=)[^&\s"']+/gi;

export function scrubString(val: string): string {
  if (!val) return val;
  let scrubbed = val;
  scrubbed = scrubbed.replace(STELLAR_SEED_PATTERN, "[Filtered]");
  scrubbed = scrubbed.replace(JWT_PATTERN, "[Filtered]");
  scrubbed = scrubbed.replace(BEARER_PATTERN, "$1[Filtered]");
  scrubbed = scrubbed.replace(SENSITIVE_QUERY_PARAM_PATTERN, "$1[Filtered]");
  return scrubbed;
}

export function isSensitiveKey(key: string): boolean {
  return SENSITIVE_KEY_PATTERN.test(key);
}

export function scrub(value: unknown): unknown {
  if (typeof value === "string") {
    return scrubString(value);
  }
  if (Array.isArray(value)) {
    return value.map(scrub);
  }
  if (!value || typeof value !== "object") {
    return value;
  }

  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>).map(([key, item]) => [
      key,
      isSensitiveKey(key) ? "[Filtered]" : scrub(item),
    ])
  );
}

export function scrubEvent<T>(event: T): T {
  return scrub(event) as T;
}
