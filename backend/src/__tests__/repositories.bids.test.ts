import { describe, it, expect, vi, beforeEach } from 'vitest';

const { mockFindMany, mockCreate } = vi.hoisted(() => ({
  mockFindMany: vi.fn(),
  mockCreate: vi.fn(),
}));

vi.mock('../db/prisma', () => ({
  default: {
    bid: {
      findMany: mockFindMany,
      create: mockCreate,
    },
  },
}));

import * as bids from '../db/repositories/bids';

describe('bids repository', () => {
  beforeEach(() => vi.clearAllMocks());

  it('findByAuction returns bids ordered by amount descending', async () => {
    mockFindMany.mockResolvedValue([{ bidId: 1n, amountStroops: 500n }]);
    const result = await bids.findByAuction(1n);
    expect(result).toHaveLength(1);
    expect(mockFindMany).toHaveBeenCalledWith(
      expect.objectContaining({ orderBy: { amountStroops: 'desc' } })
    );
  });

  it('findByBidder returns bids with limit', async () => {
    mockFindMany.mockResolvedValue([]);
    await bids.findByBidder('GBIDDER', 5);
    expect(mockFindMany).toHaveBeenCalledWith(expect.objectContaining({ take: 5 }));
  });

  it('create calls prisma.create', async () => {
    mockCreate.mockResolvedValue({ bidId: 1n });
    await bids.create({ bidId: 1n } as any);
    expect(mockCreate).toHaveBeenCalled();
  });
});
