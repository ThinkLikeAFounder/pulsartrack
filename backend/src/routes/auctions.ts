import { Router, Request, Response } from "express";
import pool from "../config/database";
import { callReadOnly } from "../services/soroban-client";
import { CONTRACT_IDS } from "../config/stellar";
import { requireAuth, rateLimitWrite } from "../middleware/auth";
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
      const rawLimit = parseInt(req.query.limit as string);
      const limit = Math.min(Math.max(isNaN(rawLimit) ? 20 : rawLimit, 1), 100);
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
      req.log?.error({ err }, "Failed to fetch auctions");
      const details =
        process.env.NODE_ENV === "development" ? err.message : undefined;
      res.status(500).json({
        error: "Failed to fetch auctions",
        ...(details && { details }),
      });
    }
  },
);

router.post(
  "/:auctionId/bid",
  requireAuth,
  rateLimitWrite(),
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
    const client = await pool.connect();
    try {
      const address = req.stellarAddress;
      const auctionId = parseInt(req.params.auctionId as string);
      const { campaignId, amountStroops } = req.body;

      await client.query("BEGIN");

      // Lock the auction row and verify it exists and is open
      const auctionResult = await client.query(
        `SELECT publisher, floor_price_stroops, status FROM auctions WHERE auction_id = $1 FOR UPDATE`,
        [auctionId],
      );
      if (auctionResult.rows.length === 0) {
        await client.query("ROLLBACK");
        return res.status(404).json({ error: "Auction not found" });
      }
      const auction = auctionResult.rows[0];
      if (auction.status !== "Open") {
        await client.query("ROLLBACK");
        return res
          .status(400)
          .json({ error: "Auction is not open for bidding" });
      }

      // Prevent self-bidding
      if (auction.publisher === address) {
        await client.query("ROLLBACK");
        return res
          .status(403)
          .json({ error: "Cannot bid on your own auction" });
      }

      // Verify bid meets floor price
      if (amountStroops < Number(auction.floor_price_stroops)) {
        await client.query("ROLLBACK");
        return res.status(400).json({ error: "Bid below floor price" });
      }

      // Verify campaign belongs to the bidder
      const campaignResult = await client.query(
        `SELECT advertiser FROM campaigns WHERE campaign_id = $1`,
        [campaignId],
      );
      if (campaignResult.rows.length === 0) {
        await client.query("ROLLBACK");
        return res.status(404).json({ error: "Campaign not found" });
      }
      if (campaignResult.rows[0].advertiser !== address) {
        await client.query("ROLLBACK");
        return res
          .status(403)
          .json({ error: "Campaign does not belong to you" });
      }

      const { rows } = await client.query(
        `INSERT INTO bids (auction_id, bidder, campaign_id, amount_stroops)
       VALUES ($1, $2, $3, $4) RETURNING *`,
        [auctionId, address, campaignId, amountStroops],
      );

      await client.query(
        `UPDATE auctions SET bid_count = bid_count + 1 WHERE auction_id = $1`,
        [auctionId],
      );

      await client.query("COMMIT");

      res.status(201).json(rows[0]);
    } catch (err: any) {
      try {
        await client.query("ROLLBACK");
      } catch (rollbackErr: any) {
        req.log?.error({ rollbackErr }, "ROLLBACK failed");
      }
      req.log?.error({ err }, "Failed to submit bid");
      const details =
        process.env.NODE_ENV === "development" ? err.message : undefined;
      res
        .status(500)
        .json({ error: "Failed to submit bid", ...(details && { details }) });
    } finally {
      client.release();
    }
  },
);

export default router;
