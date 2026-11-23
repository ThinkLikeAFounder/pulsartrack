'use client';

/**
 * Translates the app's normal contract read/write calls into operations on
 * the simulated world (demo-store.ts) when `isDemoMode()` is on. Dispatch is
 * keyed by Soroban method name — not by contract ID — because every
 * contract ID is the same empty string until a real one is deployed.
 *
 * Method names are unique across the whole app (verified against every
 * hook in useContract.ts), so this is an unambiguous router.
 */

import type { ContractArg, ScValArg, ContractCallOptions, ContractCallResult, ReadOnlyOptions } from './soroban-client';
import { getWorld, mutateWorld, demoXlm, type DemoPublisherProfile } from './demo-store';
import { useTransactionStore, type TransactionType } from '../store/tx-store';

function arg<T>(args: ContractArg[] | undefined, index: number): T {
  const a = args?.[index] as ScValArg | undefined;
  return a?.value as T;
}

function fakeTxHash(): string {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
}

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

function getOrCreatePublisher(address: string): DemoPublisherProfile {
  const world = getWorld();
  const existing = world.publishers[address];
  if (existing) return existing;
  const profile: DemoPublisherProfile = {
    address,
    registered: true,
    verified: true,
    reputationScore: 640,
    impressionsServed: 182_430,
    earnings: demoXlm(312.5),
  };
  mutateWorld((w) => {
    w.publishers[address] = profile;
  });
  return profile;
}

/** Demo equivalent of `callReadOnly` — resolves instantly, no network call. */
export function demoReadOnly<T = unknown>(options: ReadOnlyOptions): T {
  const world = getWorld();
  const { method, args } = options;

  switch (method) {
    case 'get_campaign_count':
      return world.campaigns.length as unknown as T;

    case 'get_campaign': {
      const id = Number(arg<bigint>(args, 0));
      const c = world.campaigns.find((x) => x.id === id);
      if (!c) return null as T;
      return {
        advertiser: c.advertiser,
        title: c.title,
        content_id: c.contentId,
        budget: c.budget,
        cost_per_view: c.costPerView,
        target_views: c.targetViews,
        current_views: c.currentViews,
        daily_view_limit: c.dailyViewLimit,
        status: c.status,
      } as unknown as T;
    }

    case 'get_advertiser_stats': {
      const address = arg<string>(args, 0);
      const mine = world.campaigns.filter((c) => c.advertiser === address);
      const active = mine.filter((c) => c.status === 'Active').length;
      const totalViews = mine.reduce((s, c) => s + c.currentViews, 0n);
      const totalSpent = mine.reduce((s, c) => s + c.costPerView * c.currentViews, 0n);
      return {
        active_campaigns: active,
        total_views: totalViews,
        total_spent: totalSpent,
      } as unknown as T;
    }

    case 'get_auction_count':
      return world.auctions.length as unknown as T;

    case 'get_auction': {
      const id = Number(arg<bigint>(args, 0));
      const a = world.auctions.find((x) => x.id === id);
      if (!a) return null as T;
      return {
        auction_id: BigInt(a.id),
        publisher: a.publisher,
        impression_slot: a.impressionSlot,
        floor_price: a.floorPrice,
        reserve_price: a.reservePrice,
        start_time: BigInt(a.startTime),
        end_time: BigInt(a.endTime),
        status: a.status,
        winning_bid: a.winningBid,
        winner: a.winner,
        bid_count: a.bids.length,
      } as unknown as T;
    }

    case 'get_reputation': {
      const address = arg<string>(args, 0);
      const p = world.publishers[address];
      return { score: p?.reputationScore ?? 500 } as unknown as T;
    }

    case 'get_publisher': {
      const address = arg<string>(args, 0);
      const p = world.publishers[address];
      return {
        status: p?.registered ? 'Registered' : undefined,
        impressions_served: p?.impressionsServed ?? 0,
      } as unknown as T;
    }

    case 'get_kyc_record': {
      const address = arg<string>(args, 0);
      const p = world.publishers[address];
      return { verified: p?.verified ?? false } as unknown as T;
    }

    case 'get_publisher_balance': {
      const address = arg<string>(args, 0);
      return (world.publishers[address]?.earnings ?? 0n) as unknown as T;
    }

    case 'get_revenue_pool':
      // Mirrors the contract's default split, in basis points.
      return {
        platform_pct: 250,
        publisher_pct: 9_000,
        treasury_pct: 500,
        burn_pct: 250,
      } as unknown as T;

    case 'get_subscription': {
      if (!world.subscription) return null as T;
      return { tier: world.subscription.planName } as unknown as T;
    }

    case 'get_consent':
      return (world.consent ?? null) as unknown as T;

    case 'balance':
      return world.pulsarBalance as unknown as T;

    case 'get_proposal_count':
      return world.proposals.length as unknown as T;

    case 'get_proposal': {
      const id = Number(arg<bigint>(args, 0));
      const p = world.proposals.find((x) => x.id === id);
      if (!p) return null as T;
      return {
        title: p.title,
        description: p.description,
        proposer: p.proposer,
        votes_for: p.votesFor,
        votes_against: p.votesAgainst,
        votes_abstain: p.votesAbstain,
        end_time: BigInt(p.endTime),
        is_active: p.isActive,
      } as unknown as T;
    }

    default:
      return null as T;
  }
}

/** Demo equivalent of `callContract` — mutates the simulated world and fakes a transaction lifecycle. */
export async function demoContractCall(options: ContractCallOptions): Promise<ContractCallResult> {
  const { method, args, source } = options;
  const txHash = fakeTxHash();

  const { addTransaction, updateTransaction } = useTransactionStore.getState();
  addTransaction({
    txHash,
    type: options.txType || ('other' as TransactionType),
    status: 'pending',
    description: options.description || `${method} (demo)`,
  });

  // Small, believable delay so the pending state is actually visible.
  await delay(900 + Math.random() * 700);

  try {
    const result = applyDemoWrite(method, args, source);
    updateTransaction(txHash, { status: 'success', result });
    return { success: true, txHash, result };
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Simulated transaction failed';
    updateTransaction(txHash, { status: 'failed', error: message });
    return { success: false, txHash, error: message };
  }
}

function applyDemoWrite(method: string, args: ContractArg[] | undefined, source: string): unknown {
  switch (method) {
    case 'create_campaign': {
      const budget = arg<bigint>(args, 2);
      const costPerView = arg<bigint>(args, 3);
      const targetViews = arg<bigint>(args, 5);
      const dailyViewLimit = arg<bigint>(args, 6);

      return mutateWorld((w) => {
        const id = w.nextCampaignId++;
        w.campaigns.unshift({
          id,
          advertiser: source,
          title: `Campaign #${id}`,
          contentId: '',
          budget,
          costPerView,
          targetViews: targetViews || 0n,
          currentViews: 0n,
          dailyViewLimit: dailyViewLimit || 0n,
          status: 'Active',
          createdAt: Math.floor(Date.now() / 1000),
        });
      }).campaigns[0];
    }

    case 'create_auction': {
      const impressionSlot = arg<string>(args, 1);
      const floorPrice = arg<bigint>(args, 2);
      const reservePrice = arg<bigint>(args, 3);
      const durationSeconds = Number(arg<bigint>(args, 4) ?? 3600n);
      const t = Math.floor(Date.now() / 1000);

      return mutateWorld((w) => {
        const id = w.nextAuctionId++;
        w.auctions.unshift({
          id,
          publisher: source,
          impressionSlot: impressionSlot || `slot-${id}`,
          floorPrice,
          reservePrice: reservePrice || floorPrice,
          startTime: t,
          endTime: t + durationSeconds,
          status: 'Open',
          winningBid: null,
          winner: null,
          bids: [],
        });
        getOrCreatePublisher(source);
      }).auctions[0];
    }

    case 'place_bid': {
      const auctionId = Number(arg<bigint>(args, 1));
      const amount = arg<bigint>(args, 2);
      const campaignId = Number(arg<bigint>(args, 3));

      return mutateWorld((w) => {
        const auction = w.auctions.find((a) => a.id === auctionId);
        if (!auction) throw new Error('auction not found');
        if (auction.status !== 'Open') throw new Error('auction not open');
        if (auction.publisher === source) throw new Error('publisher cannot bid on own auction');
        if (auction.winningBid !== null && amount <= auction.winningBid) {
          throw new Error('bid too low');
        }
        if (amount < auction.floorPrice) throw new Error('bid below floor price');

        auction.bids.unshift({ bidder: source, amount, campaignId, timestamp: Math.floor(Date.now() / 1000) });
        auction.winningBid = amount;
        auction.winner = source;
      });
    }

    case 'set_consent': {
      const dataProcessing = arg<boolean>(args, 1);
      const targetedAds = arg<boolean>(args, 2);
      const analytics = arg<boolean>(args, 3);
      const thirdPartySharing = arg<boolean>(args, 4);

      return mutateWorld((w) => {
        w.consent = { dataProcessing, targetedAds, analytics, thirdPartySharing };
      }).consent;
    }

    case 'cast_vote': {
      const proposalId = Number(arg<bigint>(args, 1));
      const voteType = arg<string>(args, 2) as 'For' | 'Against' | 'Abstain';
      const votePower = arg<bigint>(args, 3);

      return mutateWorld((w) => {
        const proposal = w.proposals.find((p) => p.id === proposalId);
        if (!proposal) throw new Error('proposal not found');
        if (!proposal.isActive) throw new Error('voting period has ended');
        if (proposal.voters[source]) throw new Error('already voted on this proposal');

        if (voteType === 'For') proposal.votesFor += votePower;
        else if (voteType === 'Against') proposal.votesAgainst += votePower;
        else proposal.votesAbstain += votePower;
        proposal.voters[source] = voteType;
      });
    }

    case 'create_proposal': {
      const title = arg<string>(args, 1);
      const description = arg<string>(args, 2);
      const votingPeriodLedgers = Number(arg<bigint>(args, 3));
      const LEDGER_SECONDS = 5;

      return mutateWorld((w) => {
        const id = w.nextProposalId++;
        w.proposals.unshift({
          id,
          proposer: source,
          title,
          description,
          votesFor: 0n,
          votesAgainst: 0n,
          votesAbstain: 0n,
          endTime: Math.floor(Date.now() / 1000) + votingPeriodLedgers * LEDGER_SECONDS,
          isActive: true,
          voters: {},
        });
      }).proposals[0];
    }

    case 'subscribe': {
      const planName = arg<string>(args, 1);
      return mutateWorld((w) => {
        w.subscription = { planName, subscribedAt: Math.floor(Date.now() / 1000) };
      }).subscription;
    }

    default:
      return null;
  }
}
