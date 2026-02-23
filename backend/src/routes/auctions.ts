import { Router, Request, Response } from "express";
import pool from "../config/database";
import { callReadOnly } from "../services/soroban-client";
import { CONTRACT_IDS } from "../config/stellar";
import { requireAuth } from "../middleware/auth";
import { validate } from "../middleware/validate";

const router = Router();

router.get(
  "/",
  validate({
    query: {
      limit: { type: "number", integer: true, min: 1, max: 100 },
      status: { type: "string", maxLength: 20 },
    },
  }),
  async (req: Request, res: Response) => {
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
        winningBidXlm: r.winning_bid_stroops
          ? Number(r.winning_bid_stroops) / 1e7
          : null,
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
            "get_auction_count",
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
      res
        .status(500)
        .json({ error: "Failed to fetch auctions", details: err.message });
    }
  },
);

router.post(
  "/:auctionId/bid",
  requireAuth,
  validate({
    params: {
      auctionId: { type: "number", required: true, integer: true, min: 1 },
    },
    body: {
      campaignId: { type: "number", required: true, integer: true, min: 1 },
      amountStroops: { type: "number", required: true, integer: true, min: 1 },
    },
  }),
  async (req: Request, res: Response) => {
    try {
      const address = (req as any).stellarAddress;
      const auctionId = parseInt(req.params.auctionId as string);
      const { campaignId, amountStroops } = req.body;

      const { rows } = await pool.query(
        `INSERT INTO bids (auction_id, bidder, campaign_id, amount_stroops)
       VALUES ($1, $2, $3, $4) RETURNING *`,
        [auctionId, address, campaignId, amountStroops],
      );

      await pool.query(
        `UPDATE auctions SET bid_count = bid_count + 1 WHERE auction_id = $1`,
        [auctionId],
      );

      res.status(201).json(rows[0]);
    } catch (err: any) {
      res
        .status(500)
        .json({ error: "Failed to submit bid", details: err.message });
    }
  },
);

export default router;
