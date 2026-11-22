import { describe, it, expect, vi, beforeEach } from 'vitest';

const { mockQuery } = vi.hoisted(() => ({ mockQuery: vi.fn() }));

vi.mock('../config/database', () => ({ default: { query: mockQuery } }));

import { settleExpiredAuctions, SETTLE_EXPIRED_AUCTIONS_SQL } from './auction-settler';

describe('auction settler', () => {
  beforeEach(() => vi.clearAllMocks());

  it('returns the number of auctions settled', async () => {
    mockQuery.mockResolvedValue({
      rows: [
        { auction_id: '1', status: 'Settled', winner: 'GWINNER', winning_bid_stroops: '500' },
        { auction_id: '2', status: 'Cancelled', winner: null, winning_bid_stroops: null },
      ],
    });
    await expect(settleExpiredAuctions()).resolves.toBe(2);
    expect(mockQuery).toHaveBeenCalledWith(SETTLE_EXPIRED_AUCTIONS_SQL);
  });

  it('returns 0 when nothing has ended', async () => {
    mockQuery.mockResolvedValue({ rows: [] });
    await expect(settleExpiredAuctions()).resolves.toBe(0);
  });

  it('only targets open, ended auctions and honours the reserve price', () => {
    expect(SETTLE_EXPIRED_AUCTIONS_SQL).toContain("status = 'Open'");
    expect(SETTLE_EXPIRED_AUCTIONS_SQL).toContain('end_time <= NOW()');
    expect(SETTLE_EXPIRED_AUCTIONS_SQL).toContain('reserve_price_stroops');
    expect(SETTLE_EXPIRED_AUCTIONS_SQL).toContain('SKIP LOCKED');
  });
});
