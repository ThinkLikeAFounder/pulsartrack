import type { Campaign, CampaignStatus, Publisher } from '@/types/contracts';
import type { AdvertiserCampaign } from '@/hooks/useContract';

/**
 * Shapes the raw contract reads into the types the dashboard widgets render.
 * Fields the contracts do not store are left empty rather than invented.
 */

/** The variant name of a contract enum, however the SDK decoded it. */
export function enumTag(value: unknown): string {
  if (typeof value === 'string') return value;
  if (Array.isArray(value)) return String(value[0] ?? '');
  if (value && typeof value === 'object') return Object.keys(value)[0] ?? '';
  return '';
}

function toBigInt(value: unknown): bigint {
  if (typeof value === 'bigint') return value;
  if (typeof value === 'number' && Number.isFinite(value)) return BigInt(Math.trunc(value));
  if (typeof value === 'string' && /^-?\d+$/.test(value)) return BigInt(value);
  return BigInt(0);
}

const CAMPAIGN_STATUS: Record<string, CampaignStatus> = {
  Active: 'Active',
  Paused: 'Paused',
  Completed: 'Completed',
  Expired: 'Completed',
  Cancelled: 'Cancelled',
};

/**
 * A campaign from the orchestrator's `get_campaign`. Spent is the budget
 * minus what remains, and impressions are its counted views. The contract
 * keeps no title or click count, so the title is the id and clicks are 0.
 */
export function toCampaign(raw: AdvertiserCampaign): Campaign {
  const extra = raw as AdvertiserCampaign & { remaining_budget?: unknown; created_at?: unknown };
  const budget = toBigInt(raw.budget);
  const remaining = extra.remaining_budget === undefined ? budget : toBigInt(extra.remaining_budget);
  return {
    campaign_id: BigInt(raw.id),
    advertiser: raw.advertiser ?? '',
    title: `Campaign #${raw.id}`,
    content_id: '',
    budget,
    daily_budget: BigInt(0),
    spent: budget > remaining ? budget - remaining : BigInt(0),
    impressions: toBigInt(raw.current_views),
    clicks: BigInt(0),
    status: CAMPAIGN_STATUS[enumTag(raw.status)] ?? 'Pending',
    created_at: toBigInt(extra.created_at),
    expires_at: BigInt(0),
  };
}

/** A publisher from publisher-verification's `get_publisher`. */
export function toPublisher(
  address: string,
  record: object,
): Publisher & { reputation_score: number } {
  const raw = record as Record<string, unknown>;
  const domain = typeof raw.domain === 'string' ? raw.domain : '';
  return {
    publisher: typeof raw.owner === 'string' ? raw.owner : address,
    display_name: domain || address,
    website: domain ? (/^https?:\/\//.test(domain) ? domain : `https://${domain}`) : '',
    status: enumTag(raw.status) || 'Pending',
    tier: enumTag(raw.tier),
    earnings_total: toBigInt(raw.total_earnings),
    impressions_served: toBigInt(raw.total_impressions ?? raw.impressions_served),
    joined_at: toBigInt(raw.join_ledger),
    reputation_score: Number(raw.reputation_score ?? 0),
  };
}
