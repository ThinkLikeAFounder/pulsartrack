import { Router, Request, Response } from "express";
import {
  getAccountDetails,
  getAccountTransactions,
  getFeeStats,
} from "../services/horizon";
import { stellarConfig, CONTRACT_IDS } from "../config/stellar";
import campaignRoutes from "../routes/campaigns";
import publisherRoutes from "../routes/publishers";
import auctionRoutes from "../routes/auctions";
import analyticsRoutes from "../routes/analytics";

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
    res.status(500).json({ error: err.message });
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
    res.status(500).json({ error: err.message });
  }
});

// Account transaction history
router.get(
  "/account/:address/transactions",
  async (req: Request, res: Response) => {
    try {
      const { address } = req.params;
      const limit = Math.min(parseInt(req.query.limit as string) || 20, 200);
      const txs = await getAccountTransactions(address as string, limit);
      res.json({ transactions: txs, count: txs.length });
    } catch (err: any) {
      res.status(500).json({ error: err.message });
    }
  },
);

// List deployed contract IDs
router.get("/contracts", (_req: Request, res: Response) => {
  res.json({ contracts: CONTRACT_IDS });
});

// Domain routes
router.use("/campaigns", campaignRoutes);
router.use("/publishers", publisherRoutes);
router.use("/auctions", auctionRoutes);
router.use("/governance", analyticsRoutes);

export default router;
