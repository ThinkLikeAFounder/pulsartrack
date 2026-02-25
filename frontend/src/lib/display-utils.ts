import { stroopsToXlm, STROOPS_PER_XLM } from './stellar-config';

/**
 * Format XLM amount from stroops
 */
export function formatXlm(stroops: bigint | number, decimals = 2, suffix = true): string {
  const bStroops = BigInt(stroops);
  const wholePart = bStroops / BigInt(STROOPS_PER_XLM);
  const remainder = bStroops % BigInt(STROOPS_PER_XLM);
  const fracStr = remainder.toString().padStart(7, '0').slice(0, decimals);
  const formattedWhole = wholePart.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
  return `${formattedWhole}.${fracStr}${suffix ? ' XLM' : ''}`;
}

/**
 * Format a number as currency
 */
export function formatCurrency(
  amount: number | bigint,
  currency = 'XLM',
  decimals = 2
): string {
  const bAmount = BigInt(amount);
  const divisor = currency === 'PULSAR' ? BigInt(1e7) : BigInt(STROOPS_PER_XLM);
  const wholePart = bAmount / divisor;
  const remainder = bAmount % divisor;
  const precision = 7;
  const fracStr = remainder.toString().padStart(precision, '0').slice(0, decimals);

  const formattedWhole = wholePart.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
  return `${formattedWhole}.${fracStr} ${currency}`;
}

/**
 * Format a Stellar address for display (truncated)
 */
export function formatAddress(address: string, chars = 4): string {
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
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
  if (decimals === 0) {
    return BigInt(num).toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
  }
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
  const bAmount = BigInt(amount);
  const divisor = BigInt(1e7);
  const wholePart = bAmount / divisor;
  const remainder = bAmount % divisor;
  const fracStr = remainder.toString().padStart(7, '0').slice(0, decimals);
  return `${wholePart}.${fracStr} PULSAR`;
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
