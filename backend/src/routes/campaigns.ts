import { Router, Request, Response } from 'express';
import * as campaignsRepo from '../db/repositories/campaigns';
import { callReadOnly } from '../services/soroban-client';
import { CONTRACT_IDS } from '../config/stellar';
import { requireAuth, rateLimitWrite } from '../middleware/auth';
import { validate } from '../middleware/validate';

const router = Router();

router.get('/stats', async (_req: Request, res: Response) => {
  try {
    const stats = await campaignsRepo.getStats();

    let onChainTotal: number | null = null;
    if (CONTRACT_IDS.CAMPAIGN_ORCHESTRATOR) {
      try {
        onChainTotal = await callReadOnly(
          CONTRACT_IDS.CAMPAIGN_ORCHESTRATOR,
          'get_campaign_count'
        );
      } catch {
        // Contract unavailable, rely on DB
      }
    }

    res.json({
      total_campaigns: (onChainTotal != null && onChainTotal > 0) ? onChainTotal : stats.totalCampaigns ?? 0,
      active_campaigns: stats.activeCampaigns ?? 0,
      total_impressions: stats.totalImpressions ?? 0,
      total_clicks: stats.totalClicks ?? 0,
      total_spent_xlm: (stats.totalSpentStroops ?? 0) / 1e7,
      ...(stats._partial && { _partial: stats._partial }),
    });
  } catch (err: any) {
    _req.log?.error({ err }, 'Failed to fetch campaign stats');
    const details = process.env.NODE_ENV === 'development' ? err.message : undefined;
    res.status(500).json({ error: 'Failed to fetch campaign stats', ...(details && { details }) });
  }
});

router.post('/', requireAuth, rateLimitWrite(), validate({
  body: {
    title: { type: 'string', required: true, minLength: 1, maxLength: 200 },
    contentId: { type: 'string', required: true, minLength: 1 },
    budgetStroops: { type: 'number', required: true, integer: true, min: 1 },
    dailyBudgetStroops: { type: 'number', required: true, integer: true, min: 1 },
  },
}), async (req: Request, res: Response) => {
  try {
    const address = req.stellarAddress;
    const { title, contentId, budgetStroops, dailyBudgetStroops } = req.body;

    const campaign = await campaignsRepo.create({
      advertiser: address,
      title,
      contentId,
      budgetStroops: BigInt(budgetStroops),
      dailyBudgetStroops: BigInt(dailyBudgetStroops),
    });

    res.status(201).json(campaign);
  } catch (err: any) {
    req.log?.error({ err }, 'Failed to create campaign');
    const details = process.env.NODE_ENV === 'development' ? err.message : undefined;
    res.status(500).json({ error: 'Failed to create campaign', ...(details && { details }) });
  }
});

export default router;
