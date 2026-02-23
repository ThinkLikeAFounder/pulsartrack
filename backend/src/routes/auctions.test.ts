import { describe, it, expect, vi } from 'vitest';
import request from 'supertest';
import app from '../app';
import pool from '../config/database';
import { generateTestToken } from '../test-utils';

describe('Auction Routes', () => {
    const mockAddress = 'GB7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7';
    const token = generateTestToken(mockAddress);

    describe('GET /api/auctions', () => {
        it('should return a list of auctions', async () => {
            (pool.query as any).mockResolvedValue({
                rows: [
                    {
                        auction_id: 1,
                        publisher: 'GD7...',
                        impression_slot: 'top',
                        floor_price_stroops: '100',
                        status: 'Open',
                        start_time: new Date(),
                        end_time: new Date()
                    }
                ]
            });

            const response = await request(app).get('/api/auctions');

            expect(response.status).toBe(200);
            expect(response.body).toHaveProperty('auctions');
            expect(Array.isArray(response.body.auctions)).toBe(true);
            expect(response.body.auctions[0]).toHaveProperty('auctionId');
        });
    });

    describe('POST /api/auctions/:id/bid', () => {
        it('should submit a bid when authenticated', async () => {
            const bidData = {
                campaignId: 1,
                amountStroops: 150
            };

            (pool.query as any).mockResolvedValue({
                rows: [{
                    id: 'bid-uuid',
                    auction_id: 1,
                    bidder: mockAddress,
                    campaign_id: bidData.campaignId,
                    amount_stroops: bidData.amountStroops
                }]
            });

            const response = await request(app)
                .post('/api/auctions/1/bid')
                .set('Authorization', `Bearer ${token}`)
                .send(bidData);

            expect(response.status).toBe(201);
            expect(response.body.auction_id).toBe(1);
            expect(response.body.amount_stroops).toBe(150);
        });

        it('should return 401 when not authenticated', async () => {
            const response = await request(app)
                .post('/api/auctions/1/bid')
                .send({ campaignId: 1, amountStroops: 150 });

            expect(response.status).toBe(401);
        });
    });
});
