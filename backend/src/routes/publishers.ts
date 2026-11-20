import { Router, Request, Response } from "express";
import * as publishersRepo from "../db/repositories/publishers";
import { callReadOnly, toAddressScVal } from "../services/soroban-client";
import { CONTRACT_IDS } from "../config/stellar";
import { requireAuth, rateLimitAccount, rateLimitWrite, getAuthedAddress } from "../middleware/auth";
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

      const publishers = await publishersRepo.findMany(
        { status: "Verified" },
        limit,
      );

      const result = publishers.map((r) => ({
        address: r.address,
        displayName: r.displayName,
        tier: r.tier,
        reputationScore: r.reputationScore,
        impressionsServed: Number(r.impressionsServed),
        earningsXlm: Number(r.earningsStroops) / 1e7,
        lastActivity: r.lastActivity,
      }));

      if (CONTRACT_IDS.PUBLISHER_REPUTATION) {
        await Promise.all(result.map(async (publisher) => {
          try {
            const onChainScore = await callReadOnly(
              CONTRACT_IDS.PUBLISHER_REPUTATION,
              "get_reputation",
              [toAddressScVal(publisher.address)],
            );
            if (onChainScore != null) {
              publisher.reputationScore = onChainScore;
            }
          } catch {
            // On-chain enrichment is best-effort per publisher.
          }
        }));
      }

      res.json({ publishers: result });
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
  rateLimitAccount(),
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
      const address = getAuthedAddress(req, res);
      if (!address) return; // Response already sent with 401

      const { displayName, website } = req.body;

      const existing = await publishersRepo.findByAddress(address);
      if (existing) {
        return res.status(409).json({ error: "Publisher already registered" });
      }

      const publisher = await publishersRepo.create({
        address,
        displayName,
        website: website || null,
      });

      res.status(201).json(publisher);
    } catch (err: any) {
      // #1100 — handle unique-constraint race: two concurrent registrations
      // for the same address both pass findByAddress, then one insert fails
      // with Prisma P2002. Return 409 instead of generic 500.
      if (err?.code === "P2002") {
        return res.status(409).json({ error: "Publisher already registered" });
      }
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
