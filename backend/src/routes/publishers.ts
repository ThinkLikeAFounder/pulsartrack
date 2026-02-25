import { Router, Request, Response } from 'express';
import pool from '../config/database';
import { callReadOnly, toAddressScVal } from '../services/soroban-client';
import { CONTRACT_IDS } from '../config/stellar';
import { requireAuth } from '../middleware/auth';
import { validate } from '../middleware/validate';

const router = Router();

/**
 * @openapi
 * /api/publishers/leaderboard:
 *   get:
 *     summary: Get publisher leaderboard
 *     tags: [Publishers]
 *     parameters:
 *       - in: query
 *         name: limit
 *         required: false
 *         schema:
 *           type: integer
 *           minimum: 1
 *           maximum: 100
 *     responses:
 *       200:
 *         description: Publisher leaderboard
 *         content:
 *           application/json:
 *             schema:
 *               $ref: "#/components/schemas/PublisherLeaderboardResponse"
 *       400:
 *         description: Validation failed
 *         content:
 *           application/json:
 *             schema:
 *               $ref: "#/components/schemas/ErrorResponse"
 *       500:
 *         description: Failed to fetch leaderboard
 *         content:
 *           application/json:
 *             schema:
 *               $ref: "#/components/schemas/ErrorResponse"
 */

router.get('/leaderboard', validate({
  query: {
    limit: { type: 'number', integer: true, min: 1, max: 100 },
  },
}), async (req: Request, res: Response) => {
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

/**
 * @openapi
 * /api/publishers/register:
 *   post:
 *     summary: Register a new publisher
 *     tags: [Publishers]
 *     security:
 *       - bearerAuth: []
 *     requestBody:
 *       required: true
 *       content:
 *         application/json:
 *           schema:
 *             $ref: "#/components/schemas/PublisherRegisterRequest"
 *     responses:
 *       201:
 *         description: Publisher registered
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
 *         description: Failed to register publisher
 *         content:
 *           application/json:
 *             schema:
 *               $ref: "#/components/schemas/ErrorResponse"
 */
router.post('/register', requireAuth, validate({
  body: {
    displayName: { type: 'string', required: true, minLength: 1, maxLength: 100 },
    website: { type: 'string', maxLength: 500 },
  },
}), async (req: Request, res: Response) => {
  try {
    const address = (req as any).stellarAddress;
    const { displayName, website } = req.body;

    const { rows } = await pool.query(
      `INSERT INTO publishers (address, display_name, website)
       VALUES ($1, $2, $3) RETURNING *`,
      [address, displayName, website]
    );

    res.status(201).json(rows[0]);
  } catch (err: any) {
    res.status(500).json({ error: 'Failed to register publisher', details: err.message });
  }
});

export default router;
