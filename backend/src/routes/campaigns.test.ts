import { describe, it, expect, vi } from 'vitest';
import request from 'supertest';
import app from '../app';
import pool from '../config/database';
import { generateTestToken } from '../test-utils';

describe('Campaign Routes', () => {
    const mockAddress = 'GB7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7';
    const token = generateTestToken(mockAddress);

    describe('GET /api/campaigns/stats', () => {
        // Issue #369 — mock uses string values matching PostgreSQL bigint columns,
        // and assertions verify both the numeric type and value after conversion.
        it('should return campaign statistics with correct numeric conversions', async () => {
            (pool.query as any).mockResolvedValue({
                rows: [{
                    total_campaigns: '10',       // PostgreSQL returns bigint as string
                    active_campaigns: '5',
                    total_impressions: '1000',
                    total_clicks: '50',
                    total_spent_stroops: '100000000', // 100 000 000 stroops = 10 XLM
                }]
            });

            const response = await request(app).get('/api/campaigns/stats');

            expect(response.status).toBe(200);

            // Verify field presence
            expect(response.body).toHaveProperty('total_campaigns');
            expect(response.body).toHaveProperty('active_campaigns');
            expect(response.body).toHaveProperty('total_spent_xlm');

            // Issue #369 — explicitly assert numeric type so implicit JS coercion
            // of a string doesn't mask a missing conversion in the route handler.
            expect(typeof response.body.total_spent_xlm).toBe('number');
            expect(response.body.total_spent_xlm).toBeCloseTo(10, 5);

            // Verify zero-stroops edge case doesn't produce NaN or null
            expect(Number.isFinite(response.body.total_spent_xlm)).toBe(true);
        });

        it('should convert zero stroops to 0 XLM', async () => {
            (pool.query as any).mockResolvedValue({
                rows: [{
                    total_campaigns: '0',
                    active_campaigns: '0',
                    total_impressions: '0',
                    total_clicks: '0',
                    total_spent_stroops: '0',
                }]
            });

            const response = await request(app).get('/api/campaigns/stats');

            expect(response.status).toBe(200);
            expect(typeof response.body.total_spent_xlm).toBe('number');
            expect(response.body.total_spent_xlm).toBe(0);
        });

        it('should handle large stroops values without precision loss', async () => {
            // 1 billion XLM in stroops — tests large integer handling
            (pool.query as any).mockResolvedValue({
                rows: [{
                    total_campaigns: '1',
                    active_campaigns: '1',
                    total_impressions: '999999',
                    total_clicks: '12345',
                    total_spent_stroops: '10000000000000000', // 1 000 000 000 XLM
                }]
            });

            const response = await request(app).get('/api/campaigns/stats');

            expect(response.status).toBe(200);
            expect(typeof response.body.total_spent_xlm).toBe('number');
            expect(Number.isFinite(response.body.total_spent_xlm)).toBe(true);
        });
    });

    describe('POST /api/campaigns', () => {
        it('should create a new campaign when authenticated', async () => {
            const campaignData = {
                title: 'New Campaign',
                contentId: 'cid-456',
                budgetStroops: 50000000,
                dailyBudgetStroops: 5000000
            };

            (pool.query as any).mockResolvedValue({
                rows: [{
                    id: 'uuid-1',
                    campaign_id: 1,
                    title: campaignData.title,
                    content_id: campaignData.contentId,
                    budget_stroops: campaignData.budgetStroops,
                    daily_budget_stroops: campaignData.dailyBudgetStroops
                }]
            });

            const response = await request(app)
                .post('/api/campaigns')
                .set('Authorization', `Bearer ${token}`)
                .send(campaignData);

            expect(response.status).toBe(201);
            expect(response.body).toHaveProperty('campaign_id');
            expect(response.body.title).toBe(campaignData.title);
        });

        it('should return 401 when not authenticated', async () => {
            const response = await request(app)
                .post('/api/campaigns')
                .send({});

            expect(response.status).toBe(401);
        });

        it('should return 400 for invalid input', async () => {
            const response = await request(app)
                .post('/api/campaigns')
                .set('Authorization', `Bearer ${token}`)
                .send({ title: '' }); // Missing fields and invalid title

            expect(response.status).toBe(400);
        });
    });
});
