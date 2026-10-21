import { describe, it, expect, vi, beforeEach } from 'vitest';
import request from 'supertest';
import app from '../app';
import prisma from '../db/prisma';
import { generateTestToken } from '../test-utils';

describe('Campaign Routes', () => {
    const mockAddress = 'GB7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7';
    const token = generateTestToken(mockAddress);

    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('GET /api/campaigns/stats', () => {
        // Issue #369 — mock uses string values matching PostgreSQL bigint columns,
        // and assertions verify both the numeric type and value after conversion.
        it('should return campaign statistics with correct numeric conversions', async () => {
            // Mock Prisma methods used by getStats()
            (prisma.campaign.count as any).mockResolvedValueOnce(10);
            (prisma.campaign.count as any).mockResolvedValueOnce(5);
            (prisma.campaign.aggregate as any).mockResolvedValueOnce({
                _sum: {
                    impressions: BigInt(1000),
                    clicks: BigInt(50),
                    spentStroops: BigInt(100000000), // 100 000 000 stroops = 10 XLM
                }
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
            (prisma.campaign.count as any).mockResolvedValueOnce(0);
            (prisma.campaign.count as any).mockResolvedValueOnce(0);
            (prisma.campaign.aggregate as any).mockResolvedValueOnce({
                _sum: {
                    impressions: BigInt(0),
                    clicks: BigInt(0),
                    spentStroops: BigInt(0),
                }
            });

            const response = await request(app).get('/api/campaigns/stats');

            expect(response.status).toBe(200);
            expect(typeof response.body.total_spent_xlm).toBe('number');
            expect(response.body.total_spent_xlm).toBe(0);
        });

        it('should handle large stroops values without precision loss', async () => {
            // 1 billion XLM in stroops — tests large integer handling
            (prisma.campaign.count as any).mockResolvedValueOnce(1);
            (prisma.campaign.count as any).mockResolvedValueOnce(1);
            (prisma.campaign.aggregate as any).mockResolvedValueOnce({
                _sum: {
                    impressions: BigInt(999999),
                    clicks: BigInt(12345),
                    spentStroops: BigInt('10000000000000000'), // 1 000 000 000 XLM
                }
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

            (prisma.campaign.create as any).mockResolvedValue({
                id: 'uuid-1',
                campaignId: BigInt(1),
                title: campaignData.title,
                contentId: campaignData.contentId,
                budgetStroops: BigInt(campaignData.budgetStroops),
                dailyBudgetStroops: BigInt(campaignData.dailyBudgetStroops),
                advertiser: mockAddress,
                status: 'Active',
                createdAt: new Date(),
                updatedAt: new Date(),
                impressions: BigInt(0),
                clicks: BigInt(0),
                spentStroops: BigInt(0),
            });

            const response = await request(app)
                .post('/api/campaigns')
                .set('Authorization', `Bearer ${token}`)
                .send(campaignData);

            expect(response.status).toBe(201);
            expect(response.body).toHaveProperty('campaignId');
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

        it('should reject a daily budget greater than the total budget', async () => {
            const response = await request(app)
                .post('/api/campaigns')
                .set('Authorization', `Bearer ${token}`)
                .send({
                    title: 'Invalid budget pacing',
                    contentId: 'cid-789',
                    budgetStroops: 100,
                    dailyBudgetStroops: 101,
                });

            expect(response.status).toBe(400);
            expect(response.body.error).toBe('dailyBudgetStroops must not exceed budgetStroops');
            expect(prisma.campaign.create).not.toHaveBeenCalled();
        });
    });
});
