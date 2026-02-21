import { Router, Request, Response } from 'express';
import pool from '../config/database';
import { callReadOnly, toAddressScVal } from '../services/soroban-client';
import { CONTRACT_IDS } from '../config/stellar';

const router = Router();

router.get('/leaderboard', async (req: Request, res: Response) => {
  try {
    const limit = Math.min(parseInt(req.query.limit as string) || 20, 100);

    const { rows } = await pool.query(
      `SELECT address, display_name, tier, reputation_score,
              impressions_served, earnings_stroops, last_activity
       FROM publishers
       WHERE status = 'Verified'
       ORDER BY earnings_stroops DESC, reputation_score DESC
       LIMIT $1`,
      [limit]
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
          'get_reputation',
          [toAddressScVal(publishers[0].address)]
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
    res.status(500).json({ error: 'Failed to fetch publisher leaderboard', details: err.message });
  }
});

export default router;
