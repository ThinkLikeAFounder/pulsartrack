import { Router, Request, Response } from "express";
import { getFeeStats } from "../services/horizon";
import { stellarConfig, CONTRACT_IDS } from "../config/stellar";
import { requireAuth, authRouter, rateLimitAccount } from "../middleware/auth";
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
  } catch (_err) {
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

// List deployed contract IDs (auth required)
router.get("/contracts", requireAuth, rateLimitAccount(), (_req: Request, res: Response) => {
  res.json({ contracts: CONTRACT_IDS });
});

// Domain routes
router.use("/auth", authRouter);
router.use("/account", accountRoutes);
router.use("/campaigns", campaignRoutes);
router.use("/publishers", publisherRoutes);
router.use("/auctions", auctionRoutes);
router.use("/governance", governanceRoutes);

export default router;
