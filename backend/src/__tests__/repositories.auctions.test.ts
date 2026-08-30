import { describe, it, expect, vi, beforeEach } from 'vitest';

const { mockFindMany, mockFindUnique, mockCreate, mockUpdate } = vi.hoisted(() => ({
  mockFindMany: vi.fn(),
  mockFindUnique: vi.fn(),
  mockCreate: vi.fn(),
  mockUpdate: vi.fn(),
}));

vi.mock('../db/prisma', () => ({
  default: {
    auction: {
      findMany: mockFindMany,
      findUnique: mockFindUnique,
      create: mockCreate,
      update: mockUpdate,
    },
  },
}));

import * as auctions from '../db/repositories/auctions';

describe('auctions repository', () => {
  beforeEach(() => vi.clearAllMocks());

  it('findMany returns auctions', async () => {
    mockFindMany.mockResolvedValue([{ auctionId: 1n }]);
    const result = await auctions.findMany({ status: 'Active' }, 10);
    expect(result).toHaveLength(1);
    expect(mockFindMany).toHaveBeenCalledWith(expect.objectContaining({ take: 10 }));
  });

  it('findByAuctionId returns a single auction', async () => {
    mockFindUnique.mockResolvedValue({ auctionId: 42n });
    const result = await auctions.findByAuctionId(42n);
    expect(result?.auctionId).toBe(42n);
  });

  it('create calls prisma.create', async () => {
    mockCreate.mockResolvedValue({ auctionId: 1n });
    await auctions.create({ auctionId: 1n, status: 'Active' } as any);
    expect(mockCreate).toHaveBeenCalled();
  });

  it('incrementBidCount calls prisma.update with increment', async () => {
    mockUpdate.mockResolvedValue({ auctionId: 1n, bidCount: 1 });
    await auctions.incrementBidCount(1n);
    expect(mockUpdate).toHaveBeenCalledWith(
      expect.objectContaining({ data: { bidCount: { increment: 1 } } })
    );
  });

  it('settle updates winner and status', async () => {
    mockUpdate.mockResolvedValue({ auctionId: 1n, status: 'Settled' });
    await auctions.settle(1n, 'GWINNER', 1000n);
    expect(mockUpdate).toHaveBeenCalledWith(
      expect.objectContaining({
        data: expect.objectContaining({ status: 'Settled', winner: 'GWINNER' }),
      })
    );
  });
});
