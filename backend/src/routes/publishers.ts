import { Router, Request, Response } from "express";
import pool from "../config/database";
import { callReadOnly, toAddressScVal } from "../services/soroban-client";
import { CONTRACT_IDS } from "../config/stellar";
import { requireAuth, rateLimitWrite } from "../middleware/auth";
import { validate } from "../middleware/validate";

const router = Router();

router.get(
  "/leaderboard",
  validate({
    query: {
      limit: { type: "number", integer: true, min: 1, max: 100 },
    },
  }),
  async (req: Request, res: Response) => {
    try {
      const rawLimit = parseInt(req.query.limit as string);
      const limit = Math.min(Math.max(isNaN(rawLimit) ? 20 : rawLimit, 1), 100);

const { rows } = await pool.query(
        `SELECT address, display_name, tier, reputation_score,
              impressions_served, earnings_stroops, last_activity
        FROM publishers
        WHERE status IN ('Verified', 'Pending')
        ORDER BY earnings_stroops DESC, reputation_score DESC
        LIMIT $1`,
        [limit],
      );

      const publishers = rows.map((r) => ({
        address: r.address,
        displayName: r.display_name,
        tier: r.tier,
        reputationScore: r.reputation_score,
        impressionsServed: Number(r.impressions_served),
        earningsXlm: Number(r.earnings_stroops) / 1e7,
        lastActivity: r.last_activity,
      }));

      if (publishers.length > 0 && CONTRACT_IDS.PUBLISHER_REPUTATION) {
        try {
          const onChainScore = await callReadOnly(
            CONTRACT_IDS.PUBLISHER_REPUTATION,
            "get_reputation",
            [toAddressScVal(publishers[0].address)],
          );
          if (onChainScore != null) {
            publishers[0].reputationScore = onChainScore;
          }
        } catch {
          // On-chain enrichment is best-effort
        }
      }

      res.json({ publishers });
    } catch (err: any) {
      req.log?.error({ err }, "Failed to fetch publisher leaderboard");
      const details =
        process.env.NODE_ENV === "development" ? err.message : undefined;
      res
        .status(500)
        .json({
          error: "Failed to fetch publisher leaderboard",
          ...(details && { details }),
        });
    }
  },
);

router.post(
	"/register",
	requireAuth,
	rateLimitWrite(),
	validate({
	  body: {
	    displayName: {
	      type: "string",
        required: true,
        minLength: 1,
        maxLength: 100,
      },
      website: { type: "string", maxLength: 500, format: "url" },
    },
  }),
  async (req: Request, res: Response) => {
    try {
      const address = req.stellarAddress;
      const { displayName, website } = req.body;

      const { rows } = await pool.query(
        `INSERT INTO publishers (address, display_name, website)
       VALUES ($1, $2, $3)
       ON CONFLICT (address) DO NOTHING
       RETURNING *`,
        [address, displayName, website],
      );

      if (rows.length === 0) {
        return res.status(409).json({ error: "Publisher already registered" });
      }

      res.status(201).json(rows[0]);
    } catch (err: any) {
      req.log?.error({ err }, "Failed to register publisher");
      const details =
        process.env.NODE_ENV === "development" ? err.message : undefined;
      res
        .status(500)
        .json({
          error: "Failed to register publisher",
          ...(details && { details }),
        });
    }
  },
);

export default router;
