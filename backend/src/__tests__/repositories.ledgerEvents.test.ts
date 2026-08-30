import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockQuery = vi.fn();

vi.mock('../config/database', () => ({
  default: { query: mockQuery },
}));

import * as ledgerEvents from '../db/repositories/ledgerEvents';

describe('ledgerEvents repository', () => {
  beforeEach(() => vi.clearAllMocks());

  describe('findByContract', () => {
    it('orders by ledger sequence descending with the default limit', async () => {
      mockQuery.mockResolvedValue({ rows: [{ id: 1 }] });

      const result = await ledgerEvents.findByContract('CABC');

      expect(result).toHaveLength(1);
      expect(mockQuery).toHaveBeenCalledWith(
        expect.stringContaining('ORDER BY ledger_sequence DESC'),
        ['CABC', 50],
      );
    });

    it('honours an explicit limit', async () => {
      mockQuery.mockResolvedValue({ rows: [] });

      await ledgerEvents.findByContract('CABC', 5);

      expect(mockQuery).toHaveBeenCalledWith(
        expect.any(String),
        ['CABC', 5],
      );
    });
  });

  describe('findByType', () => {
    it('filters by event type and orders by indexed_at descending', async () => {
      mockQuery.mockResolvedValue({ rows: [] });

      await ledgerEvents.findByType('BidPlaced');

      expect(mockQuery).toHaveBeenCalledWith(
        expect.stringContaining('WHERE event_type = $1'),
        ['BidPlaced', 50],
      );
    });
  });

  describe('create', () => {
    it('uses ON CONFLICT DO NOTHING for duplicate handling', async () => {
      const input = {
        ledger_sequence: 100,
        tx_hash: 'abc123',
        event_type: 'BidPlaced',
        contract_id: 'CXYZ',
        event_data: { amount: 100 },
      };
      mockQuery.mockResolvedValue({ rows: [input] });

      await ledgerEvents.create(input);

      expect(mockQuery).toHaveBeenCalledWith(
        expect.stringContaining('ON CONFLICT (tx_hash, event_type) DO NOTHING'),
        [100, 'abc123', 'CXYZ', 'BidPlaced', '{"amount":100}'],
      );
    });

    it('returns null when insert is skipped due to conflict', async () => {
      mockQuery.mockResolvedValue({ rows: [] });

      const result = await ledgerEvents.create({
        ledger_sequence: 100,
        tx_hash: 'abc123',
        event_type: 'BidPlaced',
      });

      expect(result).toBeNull();
    });
  });

  describe('getLatestSequence', () => {
    it('returns the highest indexed ledger sequence', async () => {
      mockQuery.mockResolvedValue({ rows: [{ max_seq: 987 }] });

      const result = await ledgerEvents.getLatestSequence();

      expect(result).toBe(BigInt(987));
    });

    it('falls back to zero when no events have been indexed', async () => {
      mockQuery.mockResolvedValue({ rows: [{ max_seq: null }] });

      await expect(ledgerEvents.getLatestSequence()).resolves.toBe(BigInt(0));
    });
  });
});
