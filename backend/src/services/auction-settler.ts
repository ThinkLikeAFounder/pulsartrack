import pool from '../config/database';
import { logger } from '../lib/logger';
import { errorMessage } from '../lib/errors';

/**
 * Background job that settles auctions whose end_time has passed (#1251).
 *
 * Mirrors the auction-engine contract: once an auction has ended, the highest
 * bid wins if it meets the reserve price (status 'Settled', winner and
 * winning_bid_stroops recorded); otherwise — including when there were no bids
 * — the auction is 'Cancelled' and winner/winning bid stay null.
 *
 * Everything happens in a single statement so a crash cannot leave an auction
 * half-settled. Rows are taken with FOR UPDATE SKIP LOCKED, so a bid being
 * placed concurrently (which locks the auction row) is never raced and
 * multiple API instances can run the job safely.
 */

const DEFAULT_INTERVAL_MS = 30_000;

function resolveIntervalMs(): number {
  const parsed = parseInt(process.env.AUCTION_SETTLE_INTERVAL_MS || '', 10);
  return Number.isInteger(parsed) && parsed >= 1000 ? parsed : DEFAULT_INTERVAL_MS;
}

export const SETTLE_EXPIRED_AUCTIONS_SQL = `
  WITH expired AS (
    SELECT auction_id, reserve_price_stroops
    FROM auctions
    WHERE status = 'Open' AND end_time <= NOW()
    FOR UPDATE SKIP LOCKED
  ),
  top_bid AS (
    SELECT DISTINCT ON (b.auction_id) b.auction_id, b.bidder, b.amount_stroops
    FROM bids b
    JOIN expired e ON e.auction_id = b.auction_id
    ORDER BY b.auction_id, b.amount_stroops DESC, b.timestamp ASC
  )
  UPDATE auctions a
  SET status = CASE
        WHEN t.amount_stroops >= e.reserve_price_stroops THEN 'Settled'
        ELSE 'Cancelled'
      END,
      winner = CASE
        WHEN t.amount_stroops >= e.reserve_price_stroops THEN t.bidder
        ELSE NULL
      END,
      winning_bid_stroops = CASE
        WHEN t.amount_stroops >= e.reserve_price_stroops THEN t.amount_stroops
        ELSE NULL
      END,
      settled_at = NOW()
  FROM expired e
  LEFT JOIN top_bid t ON t.auction_id = e.auction_id
  WHERE a.auction_id = e.auction_id
  RETURNING a.auction_id, a.status, a.winner, a.winning_bid_stroops
`;

/** Settle every ended auction once. Returns the number of auctions updated. */
export async function settleExpiredAuctions(): Promise<number> {
  const { rows } = await pool.query(SETTLE_EXPIRED_AUCTIONS_SQL);
  if (rows.length > 0) {
    logger.info(
      { auctions: rows.map((r) => ({ auctionId: r.auction_id, status: r.status })) },
      `[AuctionSettler] Settled ${rows.length} ended auction(s)`,
    );
  }
  return rows.length;
}

let running = false;
let timer: ReturnType<typeof setTimeout> | null = null;

async function tick() {
  if (!running) return;
  try {
    await settleExpiredAuctions();
  } catch (err) {
    // Transient DB errors must not kill the loop — retry on the next tick.
    logger.warn({ err: errorMessage(err) }, '[AuctionSettler] Settlement run failed, will retry');
  }
  if (running) {
    timer = setTimeout(tick, resolveIntervalMs());
  }
}

export function startAuctionSettler() {
  if (running) return;
  running = true;
  logger.info('[AuctionSettler] Starting auction settler');
  void tick();
}

export function stopAuctionSettler() {
  running = false;
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
  logger.info('[AuctionSettler] Stopped');
}
