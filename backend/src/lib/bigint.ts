const STROOPS_PER_XLM = BigInt(10_000_000);

type BigIntLike = bigint | string | number | null | undefined;

/**
 * Normalise a BIGINT column value (pg returns BIGINT as a string, Prisma as a
 * bigint) to a decimal string without passing through a JS Number, so values
 * above Number.MAX_SAFE_INTEGER keep full precision in JSON responses.
 */
export function bigintToString(value: BigIntLike): string {
  if (value === null || value === undefined) return "0";
  if (typeof value === "bigint") return value.toString();
  if (typeof value === "string") {
    try {
      return BigInt(value).toString();
    } catch {
      return String(value);
    }
  }
  return Number.isInteger(value) ? BigInt(value).toString() : String(value);
}

/**
 * Convert a stroop amount to XLM. The whole/fractional split is done with
 * BigInt arithmetic so only the (small) XLM quotient and the 7-digit remainder
 * are ever converted to Number — no precision is lost on the raw stroop value.
 */
export function stroopsToXlm(value: BigIntLike): number {
  if (value === null || value === undefined) return 0;
  let stroops: bigint;
  try {
    stroops = typeof value === "bigint" ? value : BigInt(value);
  } catch {
    return Number(value) / 1e7;
  }
  const negative = stroops < BigInt(0);
  const abs = negative ? -stroops : stroops;
  const whole = Number(abs / STROOPS_PER_XLM);
  const fraction = Number(abs % STROOPS_PER_XLM) / 1e7;
  const xlm = whole + fraction;
  return negative ? -xlm : xlm;
}
