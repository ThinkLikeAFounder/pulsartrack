import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

const { mockGetServer, mockCreate, mockGetLatestSequence } = vi.hoisted(() => ({
  mockGetServer: vi.fn(),
  mockCreate: vi.fn(),
  mockGetLatestSequence: vi.fn(),
}));

vi.mock('../services/soroban-client', () => ({
  getServer: mockGetServer,
}));

vi.mock('../db/repositories/ledgerEvents', () => ({
  create: mockCreate,
  getLatestSequence: mockGetLatestSequence,
}));

vi.mock('../lib/logger', () => ({
  logger: {
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
  },
}));

import { startEventIndexer, stopEventIndexer } from '../services/event-indexer';

describe('event-indexer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockGetLatestSequence.mockResolvedValue(BigInt(0));
  });

  afterEach(() => {
    stopEventIndexer();
  });

  describe('event_index extraction from event.id', () => {
    it('extracts stable event_index from the event id field, not the loop counter', async () => {
      // Simulate two overlapping polls that return the same transaction's events
      // in different batch positions. The event_index should remain stable based
      // on the event.id field, not the array index (#1094).
      
      const sharedTxHash = 'abc123def456abc123def456abc123def456abc123def456abc123def456abcd';
      
      // First poll: returns events at array indices 0 and 1
      const firstPollEvents = [
        {
          id: '0016010972359577600-0000000000', // TOID-event_index (event 0 in tx)
          txHash: sharedTxHash,
          ledger: 100,
          contractId: { toString: () => 'CXYZ' },
          topic: ['BidPlaced'],
          value: { amount: 100 },
        },
        {
          id: '0016010972359577600-0000000001', // TOID-event_index (event 1 in tx)
          txHash: sharedTxHash,
          ledger: 100,
          contractId: { toString: () => 'CXYZ' },
          topic: ['BidAccepted'],
          value: { amount: 100 },
        },
      ];

      // Second poll: returns the same events but at different array indices
      // (e.g., after a restart or backfill where the query window shifted)
      const secondPollEvents = [
        {
          id: '0016010972359577600-0000000001', // Same event, different array position
          txHash: sharedTxHash,
          ledger: 100,
          contractId: { toString: () => 'CXYZ' },
          topic: ['BidAccepted'],
          value: { amount: 100 },
        },
        {
          id: '0016010972359577600-0000000000', // Same event, different array position
          txHash: sharedTxHash,
          ledger: 100,
          contractId: { toString: () => 'CXYZ' },
          topic: ['BidPlaced'],
          value: { amount: 100 },
        },
      ];

      const mockServer = {
        getEvents: vi.fn()
          .mockResolvedValueOnce({ events: firstPollEvents })
          .mockResolvedValueOnce({ events: secondPollEvents }),
      };
      mockGetServer.mockReturnValue(mockServer);

      // First poll writes both events
      mockCreate.mockResolvedValue({ id: '1' });

      startEventIndexer();

      // Wait for first poll to complete
      await vi.waitFor(() => {
        expect(mockCreate).toHaveBeenCalledTimes(2);
      }, { timeout: 1000 });

      // Verify first poll used stable event_index from id field (0 and 1)
      expect(mockCreate).toHaveBeenNthCalledWith(1, expect.objectContaining({
        tx_hash: sharedTxHash,
        event_index: 0, // From id '...600-0000000000', NOT array index 0
        event_type: 'BidPlaced',
      }));
      expect(mockCreate).toHaveBeenNthCalledWith(2, expect.objectContaining({
        tx_hash: sharedTxHash,
        event_index: 1, // From id '...600-0000000001', NOT array index 1
        event_type: 'BidAccepted',
      }));

      vi.clearAllMocks();
      mockCreate.mockResolvedValue(null); // Simulate conflict (duplicate detection)

      // Trigger second poll by advancing time
      await vi.advanceTimersByTimeAsync(5000);

      // Wait for second poll to complete
      await vi.waitFor(() => {
        expect(mockCreate).toHaveBeenCalledTimes(2);
      }, { timeout: 1000 });

      // Verify second poll uses the SAME stable event_index values
      // even though the events are at different array positions
      expect(mockCreate).toHaveBeenNthCalledWith(1, expect.objectContaining({
        tx_hash: sharedTxHash,
        event_index: 1, // From id '...600-0000000001', NOT array index 0
        event_type: 'BidAccepted',
      }));
      expect(mockCreate).toHaveBeenNthCalledWith(2, expect.objectContaining({
        tx_hash: sharedTxHash,
        event_index: 0, // From id '...600-0000000000', NOT array index 1
        event_type: 'BidPlaced',
      }));

      stopEventIndexer();
    });

    it('handles events without an id field by defaulting to 0', async () => {
      const mockServer = {
        getEvents: vi.fn().mockResolvedValue({
          events: [
            {
              // Missing id field (defensive handling for SDK changes)
              txHash: 'malformed-event',
              ledger: 200,
              contractId: { toString: () => 'CABC' },
              topic: ['SystemEvent'],
              value: null,
            },
          ],
        }),
      };
      mockGetServer.mockReturnValue(mockServer);
      mockCreate.mockResolvedValue({ id: '1' });

      startEventIndexer();

      await vi.waitFor(() => {
        expect(mockCreate).toHaveBeenCalledTimes(1);
      }, { timeout: 1000 });

      expect(mockCreate).toHaveBeenCalledWith(expect.objectContaining({
        event_index: 0, // Falls back to 0 when id is missing
      }));

      stopEventIndexer();
    });

    it('handles malformed id fields by defaulting to 0', async () => {
      const mockServer = {
        getEvents: vi.fn().mockResolvedValue({
          events: [
            {
              id: 'invalid-format', // No hyphen separator
              txHash: 'malformed-id-event',
              ledger: 300,
              contractId: { toString: () => 'CDEF' },
              topic: ['UnknownEvent'],
              value: {},
            },
          ],
        }),
      };
      mockGetServer.mockReturnValue(mockServer);
      mockCreate.mockResolvedValue({ id: '1' });

      startEventIndexer();

      await vi.waitFor(() => {
        expect(mockCreate).toHaveBeenCalledTimes(1);
      }, { timeout: 1000 });

      expect(mockCreate).toHaveBeenCalledWith(expect.objectContaining({
        event_index: 0, // Falls back to 0 when id format is invalid
      }));

      stopEventIndexer();
    });
  });
});
