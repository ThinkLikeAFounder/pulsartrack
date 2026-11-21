import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { demoReadOnly, demoContractCall } from './demo-handlers';
import { getWorld, resetDemoWorld, demoXlm } from './demo-store';
import {
  addressToScVal,
  boolToScVal,
  i128ToScVal,
  stringToScVal,
  u32ToScVal,
  u64ToScVal,
  type ContractArg,
  type ContractCallResult,
} from './soroban-client';
import { useTransactionStore } from '../store/tx-store';

// Addresses used only inside the simulated world; not real accounts.
const ME = 'GDEMOUSERAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';
const OTHER = 'GDEMOOTHERBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB';

/** Runs a demo write, fast-forwarding through its simulated network delay. */
async function write(
  method: string,
  args: ContractArg[],
  source = ME,
): Promise<ContractCallResult> {
  const pending = demoContractCall({ contractId: '', method, args, source });
  await vi.advanceTimersByTimeAsync(2_000);
  return pending;
}

function read<T = unknown>(method: string, args: ContractArg[] = []): T {
  return demoReadOnly<T>({ contractId: '', method, args });
}

// Same argument order the real hooks in useContract.ts send.
const createCampaignArgs = (budgetXlm: number, cpvXlm: number, target: number, daily: number) => [
  addressToScVal(ME),
  u32ToScVal(1),
  i128ToScVal(demoXlm(budgetXlm)),
  i128ToScVal(demoXlm(cpvXlm)),
  u32ToScVal(30 * 17_280),
  u64ToScVal(target),
  u64ToScVal(daily),
  boolToScVal(true),
];

const placeBidArgs = (auctionId: number, amount: bigint, campaignId = 1, source = ME) => [
  addressToScVal(source),
  u64ToScVal(auctionId),
  i128ToScVal(amount),
  u64ToScVal(campaignId),
];

const castVoteArgs = (proposalId: number, vote: 'For' | 'Against' | 'Abstain', power: bigint) => [
  addressToScVal(ME),
  u64ToScVal(proposalId),
  stringToScVal(vote),
  i128ToScVal(power),
];

describe('demo-handlers', () => {
  beforeEach(() => {
    vi.useFakeTimers({ toFake: ['setTimeout'] });
    window.localStorage.clear();
    resetDemoWorld();
    useTransactionStore.setState({ transactions: [] });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('demoReadOnly', () => {
    it('reads seeded counts', () => {
      expect(read('get_campaign_count')).toBe(1);
      expect(read('get_auction_count')).toBe(3);
      expect(read('get_proposal_count')).toBe(3);
      expect(read('balance', [addressToScVal(ME)])).toBe(demoXlm(1_250));
    });

    it('maps a campaign to the contract field names', () => {
      const campaign = read<Record<string, unknown>>('get_campaign', [u64ToScVal(1)]);
      expect(campaign).toMatchObject({
        title: 'Summer Launch — Testnet Beta',
        budget: demoXlm(5_000),
        cost_per_view: demoXlm(0.05),
        target_views: 100_000n,
        current_views: 42_318n,
        daily_view_limit: 20_000n,
        status: 'Active',
      });
      expect(typeof campaign.advertiser).toBe('string');
    });

    it('maps an auction to the contract field names', () => {
      const auction = read<Record<string, unknown>>('get_auction', [u64ToScVal(1)]);
      expect(auction).toMatchObject({
        auction_id: 1n,
        impression_slot: 'homepage-hero-banner',
        floor_price: demoXlm(2),
        status: 'Open',
        winning_bid: demoXlm(3.2),
        bid_count: 2,
      });
      expect(typeof auction.end_time).toBe('bigint');
    });

    it('maps a proposal to the contract field names', () => {
      const proposal = read<Record<string, unknown>>('get_proposal', [u64ToScVal(3)]);
      expect(proposal).toMatchObject({
        title: 'Fund a public contract-audit bounty pool',
        votes_for: demoXlm(1_900),
        votes_against: demoXlm(220),
        votes_abstain: demoXlm(60),
        is_active: false,
      });
      expect(typeof proposal.end_time).toBe('bigint');
    });

    it('returns null for unknown ids and unknown methods', () => {
      expect(read('get_campaign', [u64ToScVal(999)])).toBeNull();
      expect(read('get_auction', [u64ToScVal(999)])).toBeNull();
      expect(read('get_proposal', [u64ToScVal(999)])).toBeNull();
      expect(read('not_a_method')).toBeNull();
    });

    it('returns defaults for an unregistered publisher', () => {
      expect(read('get_reputation', [addressToScVal(ME)])).toEqual({ score: 500 });
      expect(read('get_publisher', [addressToScVal(ME)])).toEqual({
        status: undefined,
        impressions_served: 0,
      });
      expect(read('get_kyc_record', [addressToScVal(ME)])).toEqual({ verified: false });
      expect(read('get_publisher_balance', [addressToScVal(ME)])).toBe(0n);
    });

    it('starts with no subscription or consent', () => {
      expect(read('get_subscription', [addressToScVal(ME)])).toBeNull();
      expect(read('get_consent', [addressToScVal(ME)])).toBeNull();
    });
  });

  describe('create_campaign', () => {
    it('increments the count and can be read back', async () => {
      const result = await write('create_campaign', createCampaignArgs(100, 0.01, 5_000, 500));

      expect(result.success).toBe(true);
      expect(read('get_campaign_count')).toBe(2);
      const campaign = read<Record<string, unknown>>('get_campaign', [u64ToScVal(2)]);
      expect(campaign).toMatchObject({
        advertiser: ME,
        budget: demoXlm(100),
        cost_per_view: demoXlm(0.01),
        target_views: 5_000n,
        current_views: 0n,
        daily_view_limit: 500n,
        status: 'Active',
      });
      expect(getWorld().nextCampaignId).toBe(3);
    });

    it('shows up in the advertiser stats', async () => {
      await write('create_campaign', createCampaignArgs(100, 0.01, 5_000, 500));
      expect(read('get_advertiser_stats', [addressToScVal(ME)])).toEqual({
        active_campaigns: 1,
        total_views: 0n,
        total_spent: 0n,
      });
    });
  });

  describe('create_auction', () => {
    it('creates an open auction with the expected shape', async () => {
      const before = Math.floor(Date.now() / 1000);
      const result = await write('create_auction', [
        addressToScVal(ME),
        stringToScVal('footer-728x90'),
        i128ToScVal(demoXlm(1)),
        i128ToScVal(demoXlm(0.8)),
        u64ToScVal(7_200),
      ]);

      expect(result.success).toBe(true);
      expect(read('get_auction_count')).toBe(4);
      const auction = getWorld().auctions.find((a) => a.id === 4)!;
      expect(auction).toMatchObject({
        publisher: ME,
        impressionSlot: 'footer-728x90',
        floorPrice: demoXlm(1),
        reservePrice: demoXlm(0.8),
        status: 'Open',
        winningBid: null,
        winner: null,
        bids: [],
      });
      expect(auction.endTime - auction.startTime).toBe(7_200);
      expect(auction.startTime).toBeGreaterThanOrEqual(before);
    });

    it('registers the creator as a publisher', async () => {
      await write('create_auction', [
        addressToScVal(ME),
        stringToScVal('footer-728x90'),
        i128ToScVal(demoXlm(1)),
        i128ToScVal(demoXlm(1)),
        u64ToScVal(3_600),
      ]);

      expect(read('get_publisher', [addressToScVal(ME)])).toMatchObject({ status: 'Registered' });
      expect(read('get_kyc_record', [addressToScVal(ME)])).toEqual({ verified: true });
      expect(read('get_reputation', [addressToScVal(ME)])).toEqual({ score: 640 });
      expect(read('get_publisher_balance', [addressToScVal(ME)])).toBe(demoXlm(312.5));
    });
  });

  describe('place_bid', () => {
    it('accepts a bid above the floor and makes it the winning bid', async () => {
      // Auction 2 is open with a 0.75 XLM floor and no bids.
      const result = await write('place_bid', placeBidArgs(2, demoXlm(1)));

      expect(result.success).toBe(true);
      const auction = read<Record<string, unknown>>('get_auction', [u64ToScVal(2)]);
      expect(auction).toMatchObject({ winning_bid: demoXlm(1), winner: ME, bid_count: 1 });
    });

    it('accepts a bid exactly at the floor', async () => {
      const result = await write('place_bid', placeBidArgs(2, demoXlm(0.75)));
      expect(result.success).toBe(true);
    });

    it('rejects a bid below the floor price', async () => {
      const result = await write('place_bid', placeBidArgs(2, demoXlm(0.5)));
      expect(result).toMatchObject({ success: false, error: 'bid below floor price' });
      expect(getWorld().auctions.find((a) => a.id === 2)!.bids).toHaveLength(0);
    });

    it('rejects a bid that does not beat the highest bid', async () => {
      // Auction 1's highest bid is 3.2 XLM.
      const equal = await write('place_bid', placeBidArgs(1, demoXlm(3.2)));
      const lower = await write('place_bid', placeBidArgs(1, demoXlm(3)));

      expect(equal).toMatchObject({ success: false, error: 'bid too low' });
      expect(lower).toMatchObject({ success: false, error: 'bid too low' });
      expect(getWorld().auctions.find((a) => a.id === 1)!.winningBid).toBe(demoXlm(3.2));
    });

    it('requires each new bid to be strictly higher', async () => {
      await write('place_bid', placeBidArgs(2, demoXlm(1)), OTHER);
      const outbid = await write('place_bid', placeBidArgs(2, demoXlm(1.1)));

      expect(outbid.success).toBe(true);
      const auction = getWorld().auctions.find((a) => a.id === 2)!;
      expect(auction.winner).toBe(ME);
      expect(auction.bids.map((b) => b.amount)).toEqual([demoXlm(1.1), demoXlm(1)]);
    });

    it('rejects bids on a settled auction', async () => {
      const result = await write('place_bid', placeBidArgs(3, demoXlm(10)));
      expect(result).toMatchObject({ success: false, error: 'auction not open' });
    });

    it('rejects bids on a missing auction', async () => {
      const result = await write('place_bid', placeBidArgs(999, demoXlm(10)));
      expect(result).toMatchObject({ success: false, error: 'auction not found' });
    });

    it('rejects a publisher bidding on their own auction', async () => {
      await write('create_auction', [
        addressToScVal(ME),
        stringToScVal('own-slot'),
        i128ToScVal(demoXlm(1)),
        i128ToScVal(demoXlm(1)),
        u64ToScVal(3_600),
      ]);
      const result = await write('place_bid', placeBidArgs(4, demoXlm(5)));
      expect(result).toMatchObject({
        success: false,
        error: 'publisher cannot bid on own auction',
      });
    });
  });

  describe('cast_vote', () => {
    it('adds vote power to the chosen tally', async () => {
      const before = getWorld().proposals.find((p) => p.id === 2)!;
      const forBefore = before.votesFor;
      const againstBefore = before.votesAgainst;

      const result = await write('cast_vote', castVoteArgs(2, 'Against', demoXlm(25)));

      expect(result.success).toBe(true);
      const proposal = read<Record<string, unknown>>('get_proposal', [u64ToScVal(2)]);
      expect(proposal.votes_against).toBe(againstBefore + demoXlm(25));
      expect(proposal.votes_for).toBe(forBefore);
      expect(getWorld().proposals.find((p) => p.id === 2)!.voters[ME]).toBe('Against');
    });

    it('counts For and Abstain votes', async () => {
      const p1 = getWorld().proposals.find((p) => p.id === 1)!;
      const forBefore = p1.votesFor;
      await write('cast_vote', castVoteArgs(1, 'For', demoXlm(5)));
      expect(getWorld().proposals.find((p) => p.id === 1)!.votesFor).toBe(forBefore + demoXlm(5));

      const abstainBefore = getWorld().proposals.find((p) => p.id === 2)!.votesAbstain;
      await write('cast_vote', castVoteArgs(2, 'Abstain', demoXlm(2)));
      expect(getWorld().proposals.find((p) => p.id === 2)!.votesAbstain).toBe(
        abstainBefore + demoXlm(2),
      );
    });

    it('rejects a second vote from the same address', async () => {
      await write('cast_vote', castVoteArgs(2, 'For', demoXlm(10)));
      const forAfterFirst = getWorld().proposals.find((p) => p.id === 2)!.votesFor;

      const second = await write('cast_vote', castVoteArgs(2, 'Against', demoXlm(10)));

      expect(second).toMatchObject({ success: false, error: 'already voted on this proposal' });
      const proposal = getWorld().proposals.find((p) => p.id === 2)!;
      expect(proposal.votesFor).toBe(forAfterFirst);
      expect(proposal.voters[ME]).toBe('For');
    });

    it('rejects votes on a closed proposal', async () => {
      const result = await write('cast_vote', castVoteArgs(3, 'For', demoXlm(1)));
      expect(result).toMatchObject({ success: false, error: 'voting period has ended' });
    });

    it('rejects votes on a missing proposal', async () => {
      const result = await write('cast_vote', castVoteArgs(999, 'For', demoXlm(1)));
      expect(result).toMatchObject({ success: false, error: 'proposal not found' });
    });
  });

  describe('other writes', () => {
    it('create_proposal adds an active proposal with no votes', async () => {
      await write('create_proposal', [
        addressToScVal(ME),
        stringToScVal('Raise the floor'),
        stringToScVal('Details'),
        u64ToScVal(7 * 17_280),
      ]);

      expect(read('get_proposal_count')).toBe(4);
      expect(read('get_proposal', [u64ToScVal(4)])).toMatchObject({
        title: 'Raise the floor',
        description: 'Details',
        proposer: ME,
        votes_for: 0n,
        votes_against: 0n,
        votes_abstain: 0n,
        is_active: true,
      });
    });

    it('subscribe is visible to get_subscription', async () => {
      await write('subscribe', [addressToScVal(ME), stringToScVal('Pro'), i128ToScVal(demoXlm(50))]);
      expect(read('get_subscription', [addressToScVal(ME)])).toEqual({ tier: 'Pro' });
    });

    it('set_consent is visible to get_consent', async () => {
      await write('set_consent', [
        addressToScVal(ME),
        boolToScVal(true),
        boolToScVal(false),
        boolToScVal(true),
        boolToScVal(false),
        u64ToScVal(0),
      ]);
      expect(read('get_consent', [addressToScVal(ME)])).toEqual({
        dataProcessing: true,
        targetedAds: false,
        analytics: true,
        thirdPartySharing: false,
      });
    });
  });

  describe('transaction lifecycle', () => {
    it('records a pending tx that resolves to success', async () => {
      const pending = demoContractCall({
        contractId: '',
        method: 'place_bid',
        args: placeBidArgs(2, demoXlm(1)),
        source: ME,
      });
      expect(useTransactionStore.getState().transactions[0].status).toBe('pending');

      await vi.advanceTimersByTimeAsync(2_000);
      const result = await pending;

      const tx = useTransactionStore.getState().getTransaction(result.txHash!);
      expect(tx?.status).toBe('success');
      expect(result.txHash).toMatch(/^[0-9a-f]{64}$/);
    });

    it('records the validation error on a failed tx', async () => {
      const result = await write('place_bid', placeBidArgs(2, demoXlm(0.1)));
      const tx = useTransactionStore.getState().getTransaction(result.txHash!);
      expect(tx).toMatchObject({ status: 'failed', error: 'bid below floor price' });
    });
  });
});
