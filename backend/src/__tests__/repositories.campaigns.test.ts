import { describe, it, expect, vi, beforeEach } from 'vitest';

const { mockFindMany, mockFindUnique, mockCount, mockAggregate, mockCreate, mockUpdate } = vi.hoisted(() => ({
  mockFindMany: vi.fn(),
  mockFindUnique: vi.fn(),
  mockCount: vi.fn(),
  mockAggregate: vi.fn(),
  mockCreate: vi.fn(),
  mockUpdate: vi.fn(),
}));

vi.mock('../db/prisma', () => ({
  default: {
    campaign: {
      findMany: mockFindMany,
      findUnique: mockFindUnique,
      count: mockCount,
      aggregate: mockAggregate,
      create: mockCreate,
      update: mockUpdate,
    },
  },
}));

import * as campaigns from '../db/repositories/campaigns';

describe('campaigns repository', () => {
  beforeEach(() => vi.clearAllMocks());

  it('findMany returns campaigns', async () => {
    mockFindMany.mockResolvedValue([{ campaignId: 1n }]);
    const result = await campaigns.findMany({ status: 'Active' }, 5);
    expect(result).toHaveLength(1);
    expect(mockFindMany).toHaveBeenCalledWith(expect.objectContaining({ take: 5 }));
  });

  it('findByCampaignId returns a single campaign', async () => {
    mockFindUnique.mockResolvedValue({ campaignId: 42n });
    const result = await campaigns.findByCampaignId(42n);
    expect(result?.campaignId).toBe(42n);
  });

  it('getStats aggregates campaign data', async () => {
    mockCount.mockResolvedValueOnce(10).mockResolvedValueOnce(3);
    mockAggregate.mockResolvedValue({
      _sum: { impressions: 1000n, clicks: 50n, spentStroops: 5000n },
    });
    const stats = await campaigns.getStats();
    expect(stats.totalCampaigns).toBe(10);
    expect(stats.activeCampaigns).toBe(3);
    expect(stats.totalImpressions).toBe(1000);
    expect(stats._partial).toBeUndefined();
  });

  it('getStats handles partial failures gracefully', async () => {
    mockCount.mockRejectedValueOnce(new Error('db down'));
    mockCount.mockResolvedValueOnce(3);
    mockAggregate.mockResolvedValue({
      _sum: { impressions: 100n, clicks: 10n, spentStroops: 1000n },
    });
    const stats = await campaigns.getStats();
    expect(stats.totalCampaigns).toBeNull();
    expect(stats.activeCampaigns).toBe(3);
    expect(stats._partial?.total).toBe(true);
  });

  it('create calls prisma.create', async () => {
    mockCreate.mockResolvedValue({ campaignId: 1n });
    await campaigns.create({ campaignId: 1n } as any);
    expect(mockCreate).toHaveBeenCalled();
  });

  it('updateStatus calls prisma.update', async () => {
    mockUpdate.mockResolvedValue({ campaignId: 1n, status: 'Paused' });
    await campaigns.updateStatus(1n, 'Paused');
    expect(mockUpdate).toHaveBeenCalledWith(
      expect.objectContaining({ data: { status: 'Paused' } })
    );
  });
});
