-- PulsarTrack Database Schema
-- PostgreSQL schema for off-chain indexing of Stellar/Soroban events

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================
-- Campaigns (indexed from on-chain events)
-- ============================================================
CREATE TABLE IF NOT EXISTS campaigns (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  campaign_id BIGINT NOT NULL UNIQUE,
  advertiser VARCHAR(64) NOT NULL,
  title TEXT NOT NULL,
  content_id TEXT NOT NULL,
  budget_stroops BIGINT NOT NULL DEFAULT 0,
  daily_budget_stroops BIGINT NOT NULL DEFAULT 0,
  spent_stroops BIGINT NOT NULL DEFAULT 0,
  impressions BIGINT NOT NULL DEFAULT 0,
  clicks BIGINT NOT NULL DEFAULT 0,
  status VARCHAR(20) NOT NULL DEFAULT 'Active',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ,
  tx_hash VARCHAR(128),
  ledger_sequence BIGINT
);

CREATE INDEX IF NOT EXISTS idx_campaigns_advertiser ON campaigns(advertiser);
CREATE INDEX IF NOT EXISTS idx_campaigns_status ON campaigns(status);

-- ============================================================
-- Publishers (indexed from on-chain events)
-- ============================================================
CREATE TABLE IF NOT EXISTS publishers (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  address VARCHAR(64) NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  website TEXT,
  status VARCHAR(20) NOT NULL DEFAULT 'Pending',
  tier VARCHAR(20) NOT NULL DEFAULT 'Bronze',
  reputation_score INT NOT NULL DEFAULT 500,
  impressions_served BIGINT NOT NULL DEFAULT 0,
  earnings_stroops BIGINT NOT NULL DEFAULT 0,
  joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_activity TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_publishers_address ON publishers(address);
CREATE INDEX IF NOT EXISTS idx_publishers_tier ON publishers(tier);

-- ============================================================
-- Auctions (indexed from on-chain events)
-- ============================================================
CREATE TABLE IF NOT EXISTS auctions (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  auction_id BIGINT NOT NULL UNIQUE,
  publisher VARCHAR(64) NOT NULL,
  impression_slot TEXT NOT NULL,
  floor_price_stroops BIGINT NOT NULL DEFAULT 0,
  reserve_price_stroops BIGINT NOT NULL DEFAULT 0,
  winning_bid_stroops BIGINT,
  winner VARCHAR(64),
  bid_count INT NOT NULL DEFAULT 0,
  status VARCHAR(20) NOT NULL DEFAULT 'Open',
  start_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  end_time TIMESTAMPTZ NOT NULL,
  settled_at TIMESTAMPTZ,
  tx_hash VARCHAR(128)
);

CREATE INDEX IF NOT EXISTS idx_auctions_publisher ON auctions(publisher);
CREATE INDEX IF NOT EXISTS idx_auctions_status ON auctions(status);

-- ============================================================
-- Bids (indexed from on-chain events)
-- ============================================================
CREATE TABLE IF NOT EXISTS bids (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  auction_id BIGINT NOT NULL,
  bidder VARCHAR(64) NOT NULL,
  campaign_id BIGINT NOT NULL,
  amount_stroops BIGINT NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  tx_hash VARCHAR(128)
);

CREATE INDEX IF NOT EXISTS idx_bids_auction ON bids(auction_id);
CREATE INDEX IF NOT EXISTS idx_bids_bidder ON bids(bidder);
CREATE INDEX IF NOT EXISTS idx_bids_amount ON bids(amount_stroops);
CREATE INDEX IF NOT EXISTS idx_bids_timestamp ON bids(timestamp);
CREATE INDEX IF NOT EXISTS idx_bids_auction_amount ON bids(auction_id, amount_stroops DESC);

-- ============================================================
-- Impressions (recorded events)
-- ============================================================
CREATE TABLE IF NOT EXISTS impressions (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  campaign_id BIGINT NOT NULL,
  publisher VARCHAR(64) NOT NULL,
  viewer_hash VARCHAR(128),
  payout_stroops BIGINT NOT NULL DEFAULT 0,
  verified BOOLEAN NOT NULL DEFAULT FALSE,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  ledger_sequence BIGINT
);

CREATE INDEX IF NOT EXISTS idx_impressions_campaign ON impressions(campaign_id);
CREATE INDEX IF NOT EXISTS idx_impressions_publisher ON impressions(publisher);
CREATE INDEX IF NOT EXISTS idx_impressions_timestamp ON impressions(timestamp);
CREATE INDEX IF NOT EXISTS idx_impressions_verified ON impressions(verified);
CREATE INDEX IF NOT EXISTS idx_impressions_campaign_time ON impressions(campaign_id, timestamp);

-- ============================================================
-- Subscriptions (indexed from on-chain events)
-- ============================================================
CREATE TABLE IF NOT EXISTS subscriptions (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  subscriber VARCHAR(64) NOT NULL,
  tier VARCHAR(20) NOT NULL,
  is_annual BOOLEAN NOT NULL DEFAULT FALSE,
  amount_paid_stroops BIGINT NOT NULL,
  started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ NOT NULL,
  auto_renew BOOLEAN NOT NULL DEFAULT TRUE,
  tx_hash VARCHAR(128)
);

CREATE INDEX IF NOT EXISTS idx_subscriptions_subscriber ON subscriptions(subscriber);
CREATE INDEX IF NOT EXISTS idx_subscriptions_expires ON subscriptions(expires_at);

-- ============================================================
-- Governance Proposals (indexed from on-chain events)
-- ============================================================
CREATE TABLE IF NOT EXISTS governance_proposals (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  proposal_id BIGINT NOT NULL UNIQUE,
  proposer VARCHAR(64) NOT NULL,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  status VARCHAR(20) NOT NULL DEFAULT 'Active',
  votes_for BIGINT NOT NULL DEFAULT 0,
  votes_against BIGINT NOT NULL DEFAULT 0,
  votes_abstain BIGINT NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  voting_ends_at TIMESTAMPTZ NOT NULL,
  executed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_proposals_status ON governance_proposals(status);
CREATE INDEX IF NOT EXISTS idx_proposals_voting_end ON governance_proposals(voting_ends_at);

-- ============================================================
-- Ledger Events (raw event log)
-- ============================================================
CREATE TABLE IF NOT EXISTS ledger_events (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  ledger_sequence BIGINT NOT NULL,
  tx_hash VARCHAR(128) NOT NULL,
  contract_id VARCHAR(64),
  event_type VARCHAR(50) NOT NULL,
  event_data JSONB,
  indexed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_events_contract ON ledger_events(contract_id);
CREATE INDEX IF NOT EXISTS idx_events_type ON ledger_events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_ledger ON ledger_events(ledger_sequence);
CREATE INDEX IF NOT EXISTS idx_events_indexed_at ON ledger_events(indexed_at);
