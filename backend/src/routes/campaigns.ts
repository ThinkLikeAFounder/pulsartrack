import { Router, Request, Response } from 'express';
import * as campaignsRepo from '../db/repositories/campaigns';
import { callReadOnly } from '../services/soroban-client';
import { CONTRACT_IDS } from '../config/stellar';
import { requireAuth, rateLimitAccount, rateLimitWrite, getAuthedAddress } from '../middleware/auth';
import { validate } from '../middleware/validate';
import pool from '../config/database';

const router = Router();

// Return observed daily events only; aggregate campaign counters cannot be
// reconstructed into historical data without inventing dates.
router.get('/:id/timeseries', async (req: Request, res: Response) => {
  const campaignId = Number(req.params.id);
  const days = Math.min(Math.max(Number(req.query.days) || 30, 1), 90);
  if (!Number.isSafeInteger(campaignId) || campaignId <= 0) {
    return res.status(400).json({ error: 'Invalid campaign id' });
  }
  try {
    const { rows } = await pool.query(
      `SELECT to_char(date_trunc('day', timestamp AT TIME ZONE 'UTC'), 'YYYY-MM-DD') AS date,
              COUNT(*)::int AS impressions
       FROM impressions
       WHERE campaign_id = $1 AND timestamp >= NOW() - ($2::int * INTERVAL '1 day')
       GROUP BY 1 ORDER BY 1`,
      [campaignId, days],
    );
    return res.json(rows.map((row) => ({ date: row.date, impressions: row.impressions, clicks: 0 })));
  } catch (err) {
    req.log?.error({ err }, 'Failed to fetch campaign timeseries');
    return res.status(500).json({ error: 'Failed to fetch campaign timeseries' });
  }
});

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

router.post('/', requireAuth, rateLimitAccount(), rateLimitWrite(), validate({
  body: {
    title: { type: 'string', required: true, minLength: 1, maxLength: 200 },
    contentId: { type: 'string', required: true, minLength: 1 },
    budgetStroops: { type: 'number', required: true, integer: true, min: 1 },
    dailyBudgetStroops: { type: 'number', required: true, integer: true, min: 1 },
  },
}), async (req: Request, res: Response) => {
  try {
    const address = getAuthedAddress(req, res);
    if (!address) return; // Response already sent with 401

    const { title, contentId, budgetStroops, dailyBudgetStroops } = req.body;
    if (dailyBudgetStroops > budgetStroops) {
      return res.status(400).json({
        error: 'dailyBudgetStroops must not exceed budgetStroops',
      });
    }

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
