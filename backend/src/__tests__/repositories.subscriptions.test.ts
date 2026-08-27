import { describe, it, expect, vi, beforeEach } from 'vitest';

const { mockFindMany, mockFindFirst, mockCreate } = vi.hoisted(() => ({
  mockFindMany: vi.fn(),
  mockFindFirst: vi.fn(),
  mockCreate: vi.fn(),
}));

vi.mock('../db/prisma', () => ({
  default: {
    subscription: {
      findMany: mockFindMany,
      findFirst: mockFindFirst,
      create: mockCreate,
    },
  },
}));

import * as subscriptions from '../db/repositories/subscriptions';

describe('subscriptions repository', () => {
  beforeEach(() => vi.clearAllMocks());

  describe('findBySubscriber', () => {
    it('returns every subscription for the subscriber, newest first', async () => {
      mockFindMany.mockResolvedValue([{ id: 1 }, { id: 2 }]);

      const result = await subscriptions.findBySubscriber('GSUB');

      expect(result).toHaveLength(2);
      expect(mockFindMany).toHaveBeenCalledWith({
        where: { subscriber: 'GSUB' },
        orderBy: { startedAt: 'desc' },
      });
    });

    it('returns an empty list when the subscriber has none', async () => {
      mockFindMany.mockResolvedValue([]);

      await expect(subscriptions.findBySubscriber('GNONE')).resolves.toEqual([]);
    });
  });

  describe('findActive', () => {
    it('only considers subscriptions expiring in the future', async () => {
      mockFindFirst.mockResolvedValue({ id: 1 });

      const result = await subscriptions.findActive('GSUB');

      expect(result).toEqual({ id: 1 });
      expect(mockFindFirst).toHaveBeenCalledWith({
        where: { subscriber: 'GSUB', expiresAt: { gt: expect.any(Date) } },
        orderBy: { expiresAt: 'desc' },
      });
    });

    it('returns null when every subscription has lapsed', async () => {
      mockFindFirst.mockResolvedValue(null);

      await expect(subscriptions.findActive('GSUB')).resolves.toBeNull();
    });
  });

  describe('create', () => {
    it('passes the input through as prisma data', async () => {
      const input = { subscriber: 'GSUB' } as never;
      mockCreate.mockResolvedValue(input);

      await subscriptions.create(input);

      expect(mockCreate).toHaveBeenCalledWith({ data: input });
    });
  });
});
