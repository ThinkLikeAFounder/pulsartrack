'use client';

/**
 * In-browser simulated contract/world state used while the real Soroban
 * contracts and backend are not yet deployed (see `isDemoMode` in
 * stellar-config.ts). Everything here lives in localStorage on the viewer's
 * own machine — nothing is sent anywhere, and no real funds or on-chain
 * state are ever touched.
 *
 * This is intentionally a plain mutable module-level object (not Zustand):
 * every mock contract call is a one-off read/write, not a React subscription,
 * so a simple get/set/persist pair is enough and keeps this file framework-free.
 */

const STROOPS_PER_XLM = 10_000_000;
const xlm = (n: number): bigint => BigInt(Math.round(n * STROOPS_PER_XLM));

export interface DemoCampaign {
  id: number;
  advertiser: string;
  title: string;
  contentId: string;
  budget: bigint;
  costPerView: bigint;
  targetViews: bigint;
  currentViews: bigint;
  dailyViewLimit: bigint;
  status: string;
  createdAt: number;
}

export interface DemoBid {
  bidder: string;
  amount: bigint;
  campaignId: number;
  timestamp: number;
}

export interface DemoAuction {
  id: number;
  publisher: string;
  impressionSlot: string;
  floorPrice: bigint;
  reservePrice: bigint;
  startTime: number;
  endTime: number;
  status: 'Open' | 'Settled' | 'Cancelled';
  winningBid: bigint | null;
  winner: string | null;
  bids: DemoBid[];
}

export interface DemoProposal {
  id: number;
  proposer: string;
  title: string;
  description: string;
  votesFor: bigint;
  votesAgainst: bigint;
  votesAbstain: bigint;
  endTime: number;
  isActive: boolean;
  voters: Record<string, 'For' | 'Against' | 'Abstain'>;
}

export interface DemoPublisherProfile {
  address: string;
  registered: boolean;
  verified: boolean;
  reputationScore: number;
  impressionsServed: number;
  earnings: bigint;
}

export interface DemoWorld {
  xlmBalance: bigint;
  pulsarBalance: bigint;
  campaigns: DemoCampaign[];
  auctions: DemoAuction[];
  proposals: DemoProposal[];
  publishers: Record<string, DemoPublisherProfile>;
  consent: {
    dataProcessing: boolean;
    targetedAds: boolean;
    analytics: boolean;
    thirdPartySharing: boolean;
  } | null;
  subscription: { planName: string; subscribedAt: number } | null;
  nextCampaignId: number;
  nextAuctionId: number;
  nextProposalId: number;
}

const STORAGE_KEY = 'pulsar-demo-world-v1';

// Fixed "background activity" addresses — other advertisers/publishers/voters
// already using the simulated protocol, so the demo never opens on an empty
// room. These are plain generated-looking Stellar addresses, not real keys.
const OTHER_PUBLISHER_A = 'GBXNET7HULA4SPZ4T6SZOQQCRDHQVQIOV4MVYZ3EQF6BBQ3PWFFXRFO';
const OTHER_PUBLISHER_B = 'GDKJPV3VYQTSAPVZ7V4SYMEB7HNBMIU65CQGTDBWNHLYLXUXXXZZQF7I';
const OTHER_ADVERTISER_A = 'GCLP6IWG3KHWJ5DRSI65VWYNT3JJQZZ5BV6FKQMHZPFFPW4ADQWHUDEM';
const OTHER_PROPOSER_A = 'GA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ';
const OTHER_PROPOSER_B = 'GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37';
const OTHER_VOTER_A = 'GCFBJ55RWVM5VC2M4PUMTCNSTLKKZUTZWKYEOSEBRDZNFXUWISKLGA3I';

const now = () => Math.floor(Date.now() / 1000);

function buildSeedWorld(): DemoWorld {
  const t = now();
  const day = 86_400;

  return {
    xlmBalance: xlm(9_500),
    pulsarBalance: xlm(1_250),
    campaigns: [
      {
        id: 1,
        advertiser: OTHER_ADVERTISER_A,
        title: 'Summer Launch — Testnet Beta',
        contentId: 'QmT8rZ6b9s3xq4p1k7mN2wF5vD9cA3eB8hJ6gR4sU1yK2z',
        budget: xlm(5_000),
        costPerView: xlm(0.05),
        targetViews: 100_000n,
        currentViews: 42_318n,
        dailyViewLimit: 20_000n,
        status: 'Active',
        createdAt: t - 6 * day,
      },
    ],
    auctions: [
      {
        id: 1,
        publisher: OTHER_PUBLISHER_A,
        impressionSlot: 'homepage-hero-banner',
        floorPrice: xlm(2),
        reservePrice: xlm(1.5),
        startTime: t - day,
        endTime: t + 2 * day,
        status: 'Open',
        winningBid: xlm(3.2),
        winner: OTHER_ADVERTISER_A,
        bids: [
          { bidder: OTHER_ADVERTISER_A, amount: xlm(3.2), campaignId: 1, timestamp: t - 3_600 },
          { bidder: OTHER_ADVERTISER_A, amount: xlm(2.4), campaignId: 1, timestamp: t - 7_200 },
        ],
      },
      {
        id: 2,
        publisher: OTHER_PUBLISHER_B,
        impressionSlot: 'article-sidebar-300x250',
        floorPrice: xlm(0.75),
        reservePrice: xlm(0.5),
        startTime: t - 3_600,
        endTime: t + 4 * 3_600,
        status: 'Open',
        winningBid: null,
        winner: null,
        bids: [],
      },
      {
        id: 3,
        publisher: OTHER_PUBLISHER_A,
        impressionSlot: 'mobile-interstitial',
        floorPrice: xlm(1.2),
        reservePrice: xlm(1),
        startTime: t - 5 * day,
        endTime: t - 3 * day,
        status: 'Settled',
        winningBid: xlm(1.9),
        winner: OTHER_ADVERTISER_A,
        bids: [{ bidder: OTHER_ADVERTISER_A, amount: xlm(1.9), campaignId: 1, timestamp: t - 4 * day }],
      },
    ],
    proposals: [
      {
        id: 1,
        proposer: OTHER_PROPOSER_A,
        title: 'Lower minimum publisher reputation for Bronze tier',
        description:
          'Proposes lowering the minimum reputation score required for a publisher to list auctions from 0 to a negative floor of -100, so new publishers with a brief bad streak are not immediately locked out of the marketplace while they build a track record.',
        votesFor: xlm(820),
        votesAgainst: xlm(140),
        votesAbstain: xlm(30),
        endTime: t + 5 * day,
        isActive: true,
        voters: { [OTHER_VOTER_A]: 'For' },
      },
      {
        id: 2,
        proposer: OTHER_PROPOSER_B,
        title: 'Increase RTB auction minimum duration to 1 hour',
        description:
          'Short auctions (under 10 minutes) are getting fewer competitive bids than longer-running ones. This proposal sets a protocol-wide minimum auction duration of 1 hour to give advertisers more time to discover and bid on open impression slots.',
        votesFor: xlm(430),
        votesAgainst: xlm(510),
        votesAbstain: xlm(10),
        endTime: t + 2 * day,
        isActive: true,
        voters: {},
      },
      {
        id: 3,
        proposer: OTHER_PROPOSER_A,
        title: 'Fund a public contract-audit bounty pool',
        description:
          'Allocates treasury funds toward an ongoing bug-bounty program for the PulsarTrack Soroban contract suite, paid out in PULSAR for verified, responsibly-disclosed findings.',
        votesFor: xlm(1_900),
        votesAgainst: xlm(220),
        votesAbstain: xlm(60),
        endTime: t - day,
        isActive: false,
        voters: { [OTHER_VOTER_A]: 'For' },
      },
    ],
    publishers: {},
    consent: null,
    subscription: null,
    nextCampaignId: 2,
    nextAuctionId: 4,
    nextProposalId: 4,
  };
}

let cached: DemoWorld | null = null;

function load(): DemoWorld {
  if (cached) return cached;
  if (typeof window === 'undefined') {
    cached = buildSeedWorld();
    return cached;
  }
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      cached = buildSeedWorld();
      return cached;
    }
    const parsed = JSON.parse(raw, (_key, value) => {
      if (typeof value === 'object' && value !== null && value.__bigint__ !== undefined) {
        return BigInt(value.__bigint__);
      }
      return value;
    }) as DemoWorld;
    cached = parsed;
    return cached;
  } catch {
    cached = buildSeedWorld();
    return cached;
  }
}

function save() {
  if (typeof window === 'undefined' || !cached) return;
  try {
    const serialized = JSON.stringify(cached, (_key, value) =>
      typeof value === 'bigint' ? { __bigint__: value.toString() } : value,
    );
    window.localStorage.setItem(STORAGE_KEY, serialized);
  } catch {
    // Storage full or unavailable — demo state just won't persist across reloads.
  }
}

export function getWorld(): DemoWorld {
  return load();
}

export function mutateWorld(fn: (world: DemoWorld) => void): DemoWorld {
  const world = load();
  fn(world);
  save();
  return world;
}

/** Resets the simulated world back to its seeded starting state. */
export function resetDemoWorld(): DemoWorld {
  cached = buildSeedWorld();
  save();
  return cached;
}

export { xlm as demoXlm };
