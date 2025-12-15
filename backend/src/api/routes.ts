import { Router, Request, Response } from 'express';
import { getAccountDetails, getAccountTransactions, getFeeStats } from '../services/horizon';
import { stellarConfig, CONTRACT_IDS } from '../config/stellar';

const router = Router();

// Health check
router.get('/health', (_req: Request, res: Response) => {
  res.json({
    status: 'ok',
    service: 'PulsarTrack API',
    network: stellarConfig.network,
    timestamp: new Date().toISOString(),
  });
});

// Stellar network info
router.get('/network', async (_req: Request, res: Response) => {
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
router.get('/account/:address', async (req: Request, res: Response) => {
  try {
    const { address } = req.params;
    const account = await getAccountDetails(address);
    if (!account) {
      return res.status(404).json({ error: 'Account not found or not funded' });
    }
    res.json(account);
  } catch (err: any) {
    res.status(500).json({ error: err.message });
  }
});

// Account transaction history
router.get('/account/:address/transactions', async (req: Request, res: Response) => {
  try {
    const { address } = req.params;
    const limit = Math.min(parseInt(req.query.limit as string) || 20, 200);
    const txs = await getAccountTransactions(address, limit);
    res.json({ transactions: txs, count: txs.length });
  } catch (err: any) {
    res.status(500).json({ error: err.message });
  }
});

// List deployed contract IDs
router.get('/contracts', (_req: Request, res: Response) => {
  res.json({ contracts: CONTRACT_IDS });
});

// Campaign stats (aggregate from DB)
router.get('/campaigns/stats', async (_req: Request, res: Response) => {
  res.json({
    total_campaigns: 0,
    active_campaigns: 0,
    total_impressions: 0,
    total_clicks: 0,
    total_spent_xlm: 0,
  });
});

// Publisher leaderboard
router.get('/publishers/leaderboard', async (_req: Request, res: Response) => {
  res.json({ publishers: [] });
});

// Auction feed
router.get('/auctions', async (_req: Request, res: Response) => {
  res.json({ auctions: [], total: 0 });
});

// Governance proposals
router.get('/governance/proposals', async (_req: Request, res: Response) => {
  res.json({ proposals: [] });
});

export default router;
