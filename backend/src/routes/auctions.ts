import { Router, Request, Response } from 'express';
import pool from '../config/database';
import { callReadOnly } from '../services/soroban-client';
import { CONTRACT_IDS } from '../config/stellar';

const router = Router();

router.get('/', async (req: Request, res: Response) => {
  try {
    const limit = Math.min(parseInt(req.query.limit as string) || 20, 100);
    const status = req.query.status as string;

    let query = `
      SELECT auction_id, publisher, impression_slot, floor_price_stroops,
             winning_bid_stroops, winner, bid_count, status, start_time, end_time
      FROM auctions
    `;
    const params: any[] = [];

    if (status) {
      params.push(status);
      query += ` WHERE status = $${params.length}`;
    }

    query += ` ORDER BY start_time DESC LIMIT $${params.length + 1}`;
    params.push(limit);

    const { rows } = await pool.query(query, params);

    const auctions = rows.map((r) => ({
      auctionId: r.auction_id,
      publisher: r.publisher,
      impressionSlot: r.impression_slot,
      floorPriceXlm: Number(r.floor_price_stroops) / 1e7,
      winningBidXlm: r.winning_bid_stroops ? Number(r.winning_bid_stroops) / 1e7 : null,
      winner: r.winner,
      bidCount: r.bid_count,
      status: r.status,
      startTime: r.start_time,
      endTime: r.end_time,
    }));

    let onChainTotal: number | null = null;
    if (CONTRACT_IDS.AUCTION_ENGINE) {
      try {
        onChainTotal = await callReadOnly(
          CONTRACT_IDS.AUCTION_ENGINE,
          'get_auction_count'
        );
      } catch {
        // Contract unavailable
      }
    }

    res.json({
      auctions,
      total: onChainTotal ?? auctions.length,
    });
  } catch (err: any) {
    res.status(500).json({ error: 'Failed to fetch auctions', details: err.message });
  }
});

export default router;
