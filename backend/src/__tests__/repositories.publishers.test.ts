import { describe, it, expect, vi, beforeEach } from 'vitest';

const { mockFindMany, mockFindUnique, mockCreate, mockUpdate } = vi.hoisted(() => ({
  mockFindMany: vi.fn(),
  mockFindUnique: vi.fn(),
  mockCreate: vi.fn(),
  mockUpdate: vi.fn(),
}));

vi.mock('../db/prisma', () => ({
  default: {
    publisher: {
      findMany: mockFindMany,
      findUnique: mockFindUnique,
      create: mockCreate,
      update: mockUpdate,
    },
  },
}));

import * as publishers from '../db/repositories/publishers';

describe('publishers repository', () => {
  beforeEach(() => vi.clearAllMocks());

  describe('findMany', () => {
    it('applies an empty where clause when no filter is given', async () => {
      mockFindMany.mockResolvedValue([{ address: 'GPUB' }]);

      const result = await publishers.findMany();

      expect(result).toHaveLength(1);
      expect(mockFindMany).toHaveBeenCalledWith({
        where: {},
        orderBy: { earningsStroops: 'desc' },
        take: 20,
      });
    });

    it('filters by status and tier together', async () => {
      mockFindMany.mockResolvedValue([]);

      await publishers.findMany({ status: 'Verified', tier: 'Gold' }, 5);

      expect(mockFindMany).toHaveBeenCalledWith({
        where: { status: 'Verified', tier: 'Gold' },
        orderBy: { earningsStroops: 'desc' },
        take: 5,
      });
    });

    it('includes only the filter keys that were supplied', async () => {
      mockFindMany.mockResolvedValue([]);

      await publishers.findMany({ tier: 'Silver' });

      expect(mockFindMany).toHaveBeenCalledWith(
        expect.objectContaining({ where: { tier: 'Silver' } }),
      );
    });
  });

  describe('findByAddress', () => {
    it('looks the publisher up by address', async () => {
      mockFindUnique.mockResolvedValue({ address: 'GPUB' });

      const result = await publishers.findByAddress('GPUB');

      expect(result).toEqual({ address: 'GPUB' });
      expect(mockFindUnique).toHaveBeenCalledWith({ where: { address: 'GPUB' } });
    });

    it('returns null when the publisher is unknown', async () => {
      mockFindUnique.mockResolvedValue(null);

      await expect(publishers.findByAddress('GNOPE')).resolves.toBeNull();
    });
  });

  describe('leaderboard', () => {
    it('returns verified publishers ranked by earnings then reputation', async () => {
      mockFindMany.mockResolvedValue([]);

      await publishers.leaderboard();

      expect(mockFindMany).toHaveBeenCalledWith({
        where: { status: 'Verified' },
        orderBy: [{ earningsStroops: 'desc' }, { reputationScore: 'desc' }],
        take: 20,
      });
    });

    it('honours an explicit limit', async () => {
      mockFindMany.mockResolvedValue([]);

      await publishers.leaderboard(3);

      expect(mockFindMany).toHaveBeenCalledWith(
        expect.objectContaining({ take: 3 }),
      );
    });
  });

  describe('create', () => {
    it('passes the input through as prisma data', async () => {
      const input = { address: 'GPUB' } as never;
      mockCreate.mockResolvedValue(input);

      await publishers.create(input);

      expect(mockCreate).toHaveBeenCalledWith({ data: input });
    });
  });

  describe('updateReputation', () => {
    it('writes the new score and stamps last activity', async () => {
      mockUpdate.mockResolvedValue({ address: 'GPUB', reputationScore: 88 });

      await publishers.updateReputation('GPUB', 88);

      expect(mockUpdate).toHaveBeenCalledWith({
        where: { address: 'GPUB' },
        data: {
          reputationScore: 88,
          lastActivity: expect.any(Date),
        },
      });
    });
  });
});
