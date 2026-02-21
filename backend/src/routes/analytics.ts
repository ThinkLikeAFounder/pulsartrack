import { Router, Request, Response } from 'express';
import pool from '../config/database';
import { callReadOnly } from '../services/soroban-client';
import { CONTRACT_IDS } from '../config/stellar';

const router = Router();

router.get('/proposals', async (req: Request, res: Response) => {
  try {
    const limit = Math.min(parseInt(req.query.limit as string) || 20, 100);
    const status = req.query.status as string;

    let query = `
      SELECT proposal_id, proposer, title, description, status,
             votes_for, votes_against, votes_abstain,
             created_at, voting_ends_at, executed_at
      FROM governance_proposals
    `;
    const params: any[] = [];

    if (status) {
      params.push(status);
      query += ` WHERE status = $${params.length}`;
    }

    query += ` ORDER BY created_at DESC LIMIT $${params.length + 1}`;
    params.push(limit);

    const { rows } = await pool.query(query, params);

    const proposals = rows.map((r) => ({
      proposalId: r.proposal_id,
      proposer: r.proposer,
      title: r.title,
      description: r.description,
      status: r.status,
      votesFor: Number(r.votes_for),
      votesAgainst: Number(r.votes_against),
      votesAbstain: Number(r.votes_abstain),
      createdAt: r.created_at,
      votingEndsAt: r.voting_ends_at,
      executedAt: r.executed_at,
    }));

    let proposalCount: number | null = null;
    if (CONTRACT_IDS.GOVERNANCE_DAO) {
      try {
        proposalCount = await callReadOnly(
          CONTRACT_IDS.GOVERNANCE_DAO,
          'get_proposal_count'
        );
      } catch {
        // DAO contract unavailable
      }
    }

    res.json({
      proposals,
      totalOnChain: proposalCount,
    });
  } catch (err: any) {
    res.status(500).json({ error: 'Failed to fetch governance proposals', details: err.message });
  }
});

export default router;
