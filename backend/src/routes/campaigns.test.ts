import { describe, it, expect, vi } from 'vitest';
import request from 'supertest';
import app from '../app';
import pool from '../config/database';
import { generateTestToken } from '../test-utils';

describe('Campaign Routes', () => {
    const mockAddress = 'GB7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7';
    const token = generateTestToken(mockAddress);

    describe('GET /api/campaigns/stats', () => {
        it('should return campaign statistics', async () => {
            (pool.query as any).mockResolvedValue({
                rows: [{
                    total_campaigns: 10,
                    active_campaigns: 5,
                    total_impressions: '1000',
                    total_clicks: '50',
                    total_spent_stroops: '100000000'
                }]
            });

            const response = await request(app).get('/api/campaigns/stats');

            expect(response.status).toBe(200);
            expect(response.body).toHaveProperty('total_campaigns');
            expect(response.body).toHaveProperty('active_campaigns');
            expect(response.body.total_spent_xlm).toBe(10);
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
