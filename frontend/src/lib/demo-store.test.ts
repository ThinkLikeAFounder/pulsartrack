import { describe, it, expect, beforeEach, vi } from 'vitest';

const STORAGE_KEY = 'pulsar-demo-world-v1';

// demo-store keeps a module-level cache, so every test gets a fresh module
// instance to observe what is actually read back from localStorage.
async function freshStore() {
  vi.resetModules();
  return import('./demo-store');
}

describe('demo-store', () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  describe('seed world', () => {
    it('seeds a populated world when nothing is stored', async () => {
      const { getWorld, demoXlm } = await freshStore();
      const world = getWorld();

      expect(world.xlmBalance).toBe(demoXlm(9_500));
      expect(world.pulsarBalance).toBe(demoXlm(1_250));
      expect(world.campaigns).toHaveLength(1);
      expect(world.auctions.map((a) => a.status)).toEqual(['Open', 'Open', 'Settled']);
      expect(world.proposals.map((p) => p.isActive)).toEqual([true, true, false]);
      expect(world.publishers).toEqual({});
      expect(world.consent).toBeNull();
      expect(world.subscription).toBeNull();
    });

    it('keeps the next-id counters ahead of the seeded records', async () => {
      const { getWorld } = await freshStore();
      const world = getWorld();

      expect(world.nextCampaignId).toBe(Math.max(...world.campaigns.map((c) => c.id)) + 1);
      expect(world.nextAuctionId).toBe(Math.max(...world.auctions.map((a) => a.id)) + 1);
      expect(world.nextProposalId).toBe(Math.max(...world.proposals.map((p) => p.id)) + 1);
    });

    it('stores money and view counts as bigints', async () => {
      const { getWorld } = await freshStore();
      const [campaign] = getWorld().campaigns;

      expect(typeof campaign.budget).toBe('bigint');
      expect(typeof campaign.costPerView).toBe('bigint');
      expect(typeof campaign.currentViews).toBe('bigint');
      expect(typeof getWorld().auctions[0].floorPrice).toBe('bigint');
      expect(typeof getWorld().proposals[0].votesFor).toBe('bigint');
    });

    it('demoXlm converts XLM to integer stroops', async () => {
      const { demoXlm } = await freshStore();
      expect(demoXlm(1)).toBe(10_000_000n);
      expect(demoXlm(0.05)).toBe(500_000n);
    });
  });

  describe('mutateWorld persistence', () => {
    it('writes bigints with the __bigint__ replacer', async () => {
      const { mutateWorld } = await freshStore();
      mutateWorld((w) => {
        w.pulsarBalance = 123_456_789_012_345_678_901n;
      });

      const raw = window.localStorage.getItem(STORAGE_KEY);
      expect(raw).not.toBeNull();
      const parsed = JSON.parse(raw!);
      expect(parsed.pulsarBalance).toEqual({ __bigint__: '123456789012345678901' });
    });

    it('round-trips bigints and mutations through a reload', async () => {
      const first = await freshStore();
      first.mutateWorld((w) => {
        w.pulsarBalance = 123_456_789_012_345_678_901n;
        w.campaigns[0].currentViews += 1n;
        w.subscription = { planName: 'Pro', subscribedAt: 42 };
      });
      const viewsAfter = first.getWorld().campaigns[0].currentViews;

      const second = await freshStore();
      const reloaded = second.getWorld();

      expect(reloaded.pulsarBalance).toBe(123_456_789_012_345_678_901n);
      expect(reloaded.campaigns[0].currentViews).toBe(viewsAfter);
      expect(typeof reloaded.auctions[0].floorPrice).toBe('bigint');
      expect(reloaded.auctions[1].winningBid).toBeNull();
      expect(reloaded.subscription).toEqual({ planName: 'Pro', subscribedAt: 42 });
    });

    it('returns the same mutated world instance it persisted', async () => {
      const { mutateWorld, getWorld } = await freshStore();
      const returned = mutateWorld((w) => {
        w.nextCampaignId = 99;
      });
      expect(returned).toBe(getWorld());
      expect(getWorld().nextCampaignId).toBe(99);
    });

    it('falls back to the seed world when stored JSON is corrupt', async () => {
      window.localStorage.setItem(STORAGE_KEY, '{not valid json');
      const { getWorld } = await freshStore();
      expect(getWorld().campaigns).toHaveLength(1);
      expect(getWorld().nextAuctionId).toBe(4);
    });

    it('keeps working in memory when localStorage writes fail', async () => {
      const { mutateWorld, getWorld } = await freshStore();
      const setItem = vi
        .spyOn(Storage.prototype, 'setItem')
        .mockImplementation(() => {
          throw new Error('QuotaExceededError');
        });

      expect(() =>
        mutateWorld((w) => {
          w.nextProposalId = 50;
        }),
      ).not.toThrow();
      expect(getWorld().nextProposalId).toBe(50);
      setItem.mockRestore();
    });
  });

  describe('resetDemoWorld', () => {
    it('restores and persists the seeded state', async () => {
      const first = await freshStore();
      first.mutateWorld((w) => {
        w.campaigns = [];
      });
      first.resetDemoWorld();
      expect(first.getWorld().campaigns).toHaveLength(1);

      const second = await freshStore();
      expect(second.getWorld().campaigns).toHaveLength(1);
    });
  });
});
