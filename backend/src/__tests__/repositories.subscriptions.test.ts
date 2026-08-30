import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockQuery = vi.fn();

vi.mock('../config/database', () => ({
  default: { query: mockQuery },
}));

import * as subscriptions from '../db/repositories/subscriptions';

describe('subscriptions repository', () => {
  beforeEach(() => vi.clearAllMocks());

  describe('findBySubscriber', () => {
    it('returns every subscription for the subscriber, newest first', async () => {
      mockQuery.mockResolvedValue({ rows: [{ id: 1 }, { id: 2 }] });

      const result = await subscriptions.findBySubscriber('GSUB');

      expect(result).toHaveLength(2);
      expect(mockQuery).toHaveBeenCalledWith(
        expect.stringContaining('ORDER BY started_at DESC'),
        ['GSUB'],
      );
    });

    it('returns an empty list when the subscriber has none', async () => {
      mockQuery.mockResolvedValue({ rows: [] });

      await expect(subscriptions.findBySubscriber('GNONE')).resolves.toEqual([]);
    });
  });

  describe('findActive', () => {
    it('only considers subscriptions expiring in the future', async () => {
      mockQuery.mockResolvedValue({ rows: [{ id: 1 }] });

      const result = await subscriptions.findActive('GSUB');

      expect(result).toEqual({ id: 1 });
      expect(mockQuery).toHaveBeenCalledWith(
        expect.stringContaining('expires_at > NOW()'),
        ['GSUB'],
      );
    });

    it('returns null when every subscription has lapsed', async () => {
      mockQuery.mockResolvedValue({ rows: [] });

      await expect(subscriptions.findActive('GSUB')).resolves.toBeNull();
    });
  });

  describe('create', () => {
    it('inserts a subscription with all provided fields', async () => {
      const input = {
        subscriber: 'GSUB',
        tier: 'Premium',
        amount_paid_stroops: 1000,
        expires_at: new Date('2027-01-01'),
      };
      mockQuery.mockResolvedValue({ rows: [{ ...input, id: 1 }] });

      await subscriptions.create(input);

      expect(mockQuery).toHaveBeenCalledWith(
        expect.stringContaining('INSERT INTO subscriptions'),
        [
          'GSUB',
          'Premium',
          false,
          1000,
          null,
          input.expires_at,
          true,
          null,
        ],
      );
    });
  });
});
