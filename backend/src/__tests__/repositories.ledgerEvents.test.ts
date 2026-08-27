import { describe, it, expect, vi, beforeEach } from 'vitest';

const { mockFindMany, mockFindFirst, mockCreate } = vi.hoisted(() => ({
  mockFindMany: vi.fn(),
  mockFindFirst: vi.fn(),
  mockCreate: vi.fn(),
}));

vi.mock('../db/prisma', () => ({
  default: {
    ledgerEvent: {
      findMany: mockFindMany,
      findFirst: mockFindFirst,
      create: mockCreate,
    },
  },
}));

import * as ledgerEvents from '../db/repositories/ledgerEvents';

describe('ledgerEvents repository', () => {
  beforeEach(() => vi.clearAllMocks());

  describe('findByContract', () => {
    it('orders by ledger sequence descending with the default limit', async () => {
      mockFindMany.mockResolvedValue([{ id: 1 }]);

      const result = await ledgerEvents.findByContract('CABC');

      expect(result).toHaveLength(1);
      expect(mockFindMany).toHaveBeenCalledWith({
        where: { contractId: 'CABC' },
        orderBy: { ledgerSequence: 'desc' },
        take: 50,
      });
    });

    it('honours an explicit limit', async () => {
      mockFindMany.mockResolvedValue([]);

      await ledgerEvents.findByContract('CABC', 5);

      expect(mockFindMany).toHaveBeenCalledWith(
        expect.objectContaining({ take: 5 }),
      );
    });
  });

  describe('findByType', () => {
    it('filters by event type and orders by indexedAt descending', async () => {
      mockFindMany.mockResolvedValue([]);

      await ledgerEvents.findByType('BidPlaced');

      expect(mockFindMany).toHaveBeenCalledWith({
        where: { eventType: 'BidPlaced' },
        orderBy: { indexedAt: 'desc' },
        take: 50,
      });
    });
  });

  describe('create', () => {
    it('passes the input through as prisma data', async () => {
      const input = { contractId: 'CABC', eventType: 'BidPlaced' } as never;
      mockCreate.mockResolvedValue(input);

      await ledgerEvents.create(input);

      expect(mockCreate).toHaveBeenCalledWith({ data: input });
    });
  });

  describe('getLatestSequence', () => {
    it('returns the highest indexed ledger sequence', async () => {
      mockFindFirst.mockResolvedValue({ ledgerSequence: 987n });

      const result = await ledgerEvents.getLatestSequence();

      expect(result).toBe(987n);
      expect(mockFindFirst).toHaveBeenCalledWith({
        orderBy: { ledgerSequence: 'desc' },
        select: { ledgerSequence: true },
      });
    });

    it('falls back to zero when no events have been indexed', async () => {
      mockFindFirst.mockResolvedValue(null);

      await expect(ledgerEvents.getLatestSequence()).resolves.toBe(BigInt(0));
    });
  });
});
