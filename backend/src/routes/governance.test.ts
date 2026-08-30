import { describe, it, expect, vi, beforeEach } from 'vitest';
import request from 'supertest';
import app from '../app';
import pool from '../config/database';
import { callReadOnly } from '../services/soroban-client';

describe('Governance Routes', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('GET /api/governance/proposals', () => {
        it('returns proposals from storage instead of an empty placeholder array', async () => {
            (pool.query as any).mockResolvedValue({
                rows: [
                    {
                        proposal_id: 7,
                        proposer: 'GABC123',
                        title: 'Increase Oracle Count',
                        description: 'Raise minimum oracle quorum to 5.',
                        status: 'Active',
                        votes_for: '42',
                        votes_against: '3',
                        votes_abstain: '1',
                        created_at: new Date('2026-03-01T00:00:00Z'),
                        voting_ends_at: new Date('2026-03-08T00:00:00Z'),
                        executed_at: null,
                    },
                ],
            });
            (callReadOnly as any).mockResolvedValue(14);

            const response = await request(app).get('/api/governance/proposals');

            expect(response.status).toBe(200);
            expect(response.body.proposals).toHaveLength(1);
            expect(response.body.proposals[0]).toMatchObject({
                proposalId: 7,
                status: 'Active',
                votesFor: 42,
                votesAgainst: 3,
                votesAbstain: 1,
            });
            expect(response.body.totalOnChain).toBeNull();
            expect(callReadOnly).not.toHaveBeenCalled();
        });

        it('applies status and limit filters when provided', async () => {
            (pool.query as any).mockResolvedValue({ rows: [] });
            (callReadOnly as any).mockResolvedValue(0);

            const response = await request(app).get(
                '/api/governance/proposals?status=Active&limit=10',
            );

            expect(response.status).toBe(200);
            expect(pool.query).toHaveBeenCalledTimes(1);

            const [sql, params] = (pool.query as any).mock.calls[0];
            expect(sql).toContain('WHERE status = $1');
            expect(sql).toContain('LIMIT $2');
            expect(params).toEqual(['Active', 10]);
        });

        it('returns 500 when the proposals query fails', async () => {
            (pool.query as any).mockRejectedValue(new Error('db unavailable'));

            const response = await request(app).get('/api/governance/proposals');

            expect(response.status).toBe(500);
            expect(response.body).toHaveProperty(
                'error',
                'Failed to fetch governance proposals',
            );
        });
    });
});