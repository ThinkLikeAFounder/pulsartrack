import { Router, Request, Response } from 'express';
import pool from '../config/database';
import { callReadOnly } from '../services/soroban-client';
import { CONTRACT_IDS } from '../config/stellar';
import { requireAuth } from '../middleware/auth';
import { validate } from '../middleware/validate';

const router = Router();

/**
 * @openapi
 * /api/campaigns/stats:
 *   get:
 *     summary: Get aggregated campaign stats
 *     tags: [Campaigns]
 *     responses:
 *       200:
 *         description: Campaign statistics
 *         content:
 *           application/json:
 *             schema:
 *               $ref: "#/components/schemas/CampaignStatsResponse"
 *       500:
 *         description: Failed to fetch campaign stats
 *         content:
 *           application/json:
 *             schema:
 *               $ref: "#/components/schemas/ErrorResponse"
 */

router.get('/stats', async (_req: Request, res: Response) => {
  try {
    const { rows } = await pool.query(`
      SELECT
        COUNT(*)::int AS total_campaigns,
        COUNT(*) FILTER (WHERE status = 'Active')::int AS active_campaigns,
        COALESCE(SUM(impressions), 0)::bigint AS total_impressions,
        COALESCE(SUM(clicks), 0)::bigint AS total_clicks,
        COALESCE(SUM(spent_stroops), 0)::bigint AS total_spent_stroops
      FROM campaigns
    `);

    const stats = rows[0];

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
      total_campaigns: onChainTotal ?? stats.total_campaigns,
      active_campaigns: stats.active_campaigns,
      total_impressions: Number(stats.total_impressions),
      total_clicks: Number(stats.total_clicks),
      total_spent_xlm: Number(stats.total_spent_stroops) / 1e7,
    });
  } catch (err: any) {
    res.status(500).json({ error: 'Failed to fetch campaign stats', details: err.message });
  }
});

/**
 * @openapi
 * /api/campaigns:
 *   post:
 *     summary: Create a new campaign
 *     tags: [Campaigns]
 *     security:
 *       - bearerAuth: []
 *     requestBody:
 *       required: true
 *       content:
 *         application/json:
 *           schema:
 *             $ref: "#/components/schemas/CampaignCreateRequest"
 *     responses:
 *       201:
 *         description: Campaign created
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *       400:
 *         description: Validation failed
 *         content:
 *           application/json:
 *             schema:
 *               $ref: "#/components/schemas/ErrorResponse"
 *       401:
 *         description: Unauthorized
 *         content:
 *           application/json:
 *             schema:
 *               $ref: "#/components/schemas/ErrorResponse"
 *       500:
 *         description: Failed to create campaign
 *         content:
 *           application/json:
 *             schema:
 *               $ref: "#/components/schemas/ErrorResponse"
 */
router.post('/', requireAuth, validate({
  body: {
    title: { type: 'string', required: true, minLength: 1, maxLength: 200 },
    contentId: { type: 'string', required: true, minLength: 1 },
    budgetStroops: { type: 'number', required: true, integer: true, min: 1 },
    dailyBudgetStroops: { type: 'number', required: true, integer: true, min: 1 },
  },
}), async (req: Request, res: Response) => {
  try {
    const address = (req as any).stellarAddress;
    const { title, contentId, budgetStroops, dailyBudgetStroops } = req.body;

    const { rows } = await pool.query(
      `INSERT INTO campaigns (campaign_id, advertiser, title, content_id, budget_stroops, daily_budget_stroops)
       VALUES ((SELECT COALESCE(MAX(campaign_id), 0) + 1 FROM campaigns), $1, $2, $3, $4, $5)
       RETURNING *`,
      [address, title, contentId, budgetStroops, dailyBudgetStroops]
    );

    res.status(201).json(rows[0]);
  } catch (err: any) {
    res.status(500).json({ error: 'Failed to create campaign', details: err.message });
  }
});

export default router;
