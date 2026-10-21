/**
 * TypeScript types matching PulsarTrack Soroban contract types
 */

export type CampaignStatus = 'Active' | 'Paused' | 'Completed' | 'Cancelled' | 'Pending';

export interface Campaign {
  campaign_id: bigint;
  advertiser: string;
  title: string;
  content_id: string;
  budget: bigint;
  daily_budget: bigint;
  spent: bigint;
  impressions: bigint;
  clicks: bigint;
  status: CampaignStatus;
  created_at: bigint;
  expires_at: bigint;
}

export interface AdContent {
  content_id: string;
  owner: string;
  ipfs_hash: string;
  title: string;
  format: string;
  status: string;
  created_at: bigint;
  impressions: bigint;
  clicks: bigint;
}

export interface Publisher {
  publisher: string;
  display_name: string;
  website: string;
  status: string;
  tier: string;
  earnings_total: bigint;
  impressions_served: bigint;
  joined_at: bigint;
}

export interface ReputationScore {
  publisher: string;
  score: number;
  total_reviews: bigint;
  positive_reviews: bigint;
  negative_reviews: bigint;
  slashes: number;
  uptime_score: number;
  quality_score: number;
  last_updated: bigint;
}

export interface Auction {
  auction_id: bigint;
  publisher: string;
  impression_slot: string;
  floor_price: bigint;
  reserve_price: bigint;
  start_time: bigint;
  end_time: bigint;
  status: 'Open' | 'Closed' | 'Settled' | 'Cancelled';
  winning_bid: bigint | null;
  winner: string | null;
  bid_count: number;
}

export interface Bid {
  bidder: string;
  amount: bigint;
  campaign_id: bigint;
  timestamp: bigint;
}

export type SubscriptionTier = 'Starter' | 'Growth' | 'Business' | 'Enterprise';

export interface Subscription {
  subscriber: string;
  tier: SubscriptionTier;
  is_annual: boolean;
  amount_paid: bigint;
  started_at: bigint;
  period_start: bigint;
  expires_at: bigint;
  auto_renew: boolean;
  campaigns_used: number;
  impressions_used: bigint;
  impressions_period_start: bigint;
}

export interface SubscriptionPlan {
  tier: SubscriptionTier;
  name: string;
  price_monthly: bigint;
  price_annual: bigint;
  max_campaigns: number;
  max_impressions_per_month: bigint;
  max_publishers: number;
  analytics_enabled: boolean;
  api_access: boolean;
}

export interface PrivacyConsent {
  user: string;
  data_processing: boolean;
  targeted_ads: boolean;
  analytics: boolean;
  third_party_sharing: boolean;
  consent_hash: string;
  consented_at: bigint;
  expires_at: bigint | null;
}

export interface TargetingConfig {
  campaign_id: bigint;
  advertiser: string;
  geographic_targets: string;
  interest_segments: string;
  excluded_segments: string;
  min_age: number;
  max_age: number;
  device_types: string;
  languages: string;
  min_reputation_score: number;
  exclude_fraud: boolean;
  require_kyc: boolean;
  max_cpm: bigint;
  created_at: bigint;
  last_updated: bigint;
}

export interface GovernanceProposal {
  proposal_id: bigint;
  proposer: string;
  title: string;
  description: string;
  status: 'Active' | 'Passed' | 'Rejected' | 'Executed' | 'Cancelled';
  votes_for: bigint;
  votes_against: bigint;
  votes_abstain: bigint;
  created_at: bigint;
  voting_ends_at: bigint;
}

export interface Identity {
  account: string;
  display_name: string;
  identity_type: string;
  verified: boolean;
  credentials_hash: string | null;
  verified_at: bigint | null;
  created_at: bigint;
}
