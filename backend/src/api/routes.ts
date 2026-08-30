import { Router, Request, Response } from "express";
import {
  getFeeStats,
  getAccountDetails,
  getAccountTransactions,
} from "../services/horizon";
import { stellarConfig, CONTRACT_IDS } from "../config/stellar";
import { requireAuth } from "../middleware/auth";
import campaignRoutes from "../routes/campaigns";
import publisherRoutes from "../routes/publishers";
import auctionRoutes from "../routes/auctions";
import governanceRoutes from "../routes/governance";
import accountRoutes from "../routes/accounts";

const router = Router();

import { runAllChecks } from "../services/health";

// Health check
router.get("/health", async (_req: Request, res: Response) => {
  try {
    const checks = await runAllChecks();
    const isOk = Object.values(checks).every((status) => status === "ok");

    res.status(isOk ? 200 : 503).json({
      status: isOk ? "ok" : "error",
      checks,
      uptime: process.uptime(),
      timestamp: new Date().toISOString(),
    });
  } catch (err) {
    res.status(503).json({
      status: "error",
      checks: {
        database: "error",
        redis: "error",
        soroban_rpc: "error",
        horizon: "error",
      },
      uptime: process.uptime(),
      timestamp: new Date().toISOString(),
    });
  }
});

// Stellar network info
router.get("/network", async (_req: Request, res: Response) => {
  try {
    const fees = await getFeeStats();
    res.json({
      network: stellarConfig.network,
      horizonUrl: stellarConfig.horizonUrl,
      sorobanRpcUrl: stellarConfig.sorobanRpcUrl,
      feeStats: fees,
    });
  } catch (err: any) {
    _req.log?.error({ err }, "Failed to fetch network info");
    const details =
      process.env.NODE_ENV === "development" ? err.message : undefined;
    res.status(500).json({
      error: "Failed to fetch network info",
      ...(details && { details }),
    });
  }
});

// Account details
router.get("/account/:address", async (req: Request, res: Response) => {
  try {
    const { address } = req.params;
    const account = await getAccountDetails(address as string);
    if (!account) {
      return res.status(404).json({ error: "Account not found or not funded" });
    }
    res.json(account);
  } catch (err: any) {
    req.log?.error({ err }, "Failed to fetch account details");
    const details =
      process.env.NODE_ENV === "development" ? err.message : undefined;
    res.status(500).json({
      error: "Failed to fetch account details",
      ...(details && { details }),
    });
  }
});

// Account transaction history
router.get(
  "/account/:address/transactions",
  async (req: Request, res: Response) => {
    try {
      const { address } = req.params;
      const rawLimit = parseInt(req.query.limit as string);
      const limit = Math.min(Math.max(isNaN(rawLimit) ? 20 : rawLimit, 1), 200);
      const txs = await getAccountTransactions(address as string, limit);
      res.json({ transactions: txs.records, count: txs.records.length });
    } catch (err: any) {
      req.log?.error({ err }, "Failed to fetch account transactions");
      const details =
        process.env.NODE_ENV === "development" ? err.message : undefined;
      res.status(500).json({
        error: "Failed to fetch account transactions",
        ...(details && { details }),
      });
    }
  },
);

// List deployed contract IDs (auth required)
router.get("/contracts", requireAuth, (_req: Request, res: Response) => {
  res.json({ contracts: CONTRACT_IDS });
});

// Domain routes
router.use("/account", accountRoutes);
router.use("/campaigns", campaignRoutes);
router.use("/publishers", publisherRoutes);
router.use("/auctions", auctionRoutes);
router.use("/governance", governanceRoutes);

export default router;
