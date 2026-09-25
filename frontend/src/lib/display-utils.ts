import { stroopsToXlm, STROOPS_PER_XLM } from './stellar-config';

/**
 * Format XLM amount from stroops to a fixed-decimal string with "XLM" suffix.
 *
 * `stroopsToXlm` returns a lossless string; this helper converts to `Number`
 * for display formatting only — no arithmetic should rely on the conversion.
 */
export function formatXlm(stroops: bigint | number, decimals = 2): string {
  const xlm = stroopsToXlm(stroops);
  return `${Number(xlm).toFixed(decimals)} XLM`;
}

/**
 * Format a number as currency
 */
export function formatCurrency(
  amount: number | bigint,
  currency = 'XLM',
  decimals = 2
): string {
  const num = Number(amount) / STROOPS_PER_XLM;
  return `${num.toLocaleString(undefined, { maximumFractionDigits: decimals })} ${currency}`;
}

/**
 * Format a Stellar address for display (truncated)
 */
export function formatAddress(address: string, chars = 4): string {
  if (!address || address.length < chars * 2 + 3) return address || '';
  return `${address.slice(0, chars + 1)}...${address.slice(-chars)}`;
}

/**
 * Format a timestamp (unix seconds) to readable date
 */
export function formatTimestamp(ts: number | bigint): string {
  return new Date(Number(ts) * 1000).toLocaleString();
}

/**
 * Format a score (0-1000) as a percentage or grade
 */
export function formatScore(score: number): string {
  if (score >= 900) return `${score} (Excellent)`;
  if (score >= 700) return `${score} (Good)`;
  if (score >= 500) return `${score} (Average)`;
  if (score >= 300) return `${score} (Poor)`;
  return `${score} (Very Poor)`;
}

/**
 * Format a number with thousands separators
 */
export function formatNumber(num: number | bigint, decimals = 0): string {
  return Number(num).toLocaleString(undefined, { maximumFractionDigits: decimals });
}

/**
 * Format duration in seconds to human-readable string
 */
export function formatDuration(seconds: number | bigint): string {
  const s = Number(seconds);
  if (s < 60) return `${s}s`;
  if (s < 3600) return `${Math.floor(s / 60)}m`;
  if (s < 86400) return `${Math.floor(s / 3600)}h`;
  if (s < 2592000) return `${Math.floor(s / 86400)}d`;
  return `${Math.floor(s / 2592000)}mo`;
}

/**
 * Format PULSAR token amount (7 decimal places)
 */
export function formatPulsar(amount: bigint | number, decimals = 2): string {
  const num = Number(amount) / 1e7;
  return `${num.toFixed(decimals)} PULSAR`;
}

/**
 * Get subscription tier label
 */
export function getTierLabel(tier: number): string {
  const tiers: Record<number, string> = {
    0: 'Starter',
    1: 'Growth',
    2: 'Business',
    3: 'Enterprise',
  };
  return tiers[tier] || 'Unknown';
}

/**
 * Get reputation tier from score
 */
export function getReputationTier(score: number): string {
  if (score >= 800) return 'Platinum';
  if (score >= 600) return 'Gold';
  if (score >= 400) return 'Silver';
  return 'Bronze';
}
