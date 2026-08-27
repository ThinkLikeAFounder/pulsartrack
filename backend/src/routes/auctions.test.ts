import { describe, it, expect, vi, beforeEach } from 'vitest';
import request from 'supertest';
import app from '../app';
import pool from '../config/database';
import { generateTestToken } from '../test-utils';

describe('Auction Routes', () => {
    const mockAddress = 'GB7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7';
    const otherAddress = 'GD7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7';
    const token = generateTestToken(mockAddress);

    beforeEach(() => {
        vi.clearAllMocks();
    });

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
        function setupClientMock(...queryResults: any[]) {
            const mockClient = {
                query: vi.fn(),
                release: vi.fn(),
            };
            // The route issues BEGIN before its first real query. Queued
            // `mockResolvedValueOnce` values are consumed in call order, so
            // without this the BEGIN would swallow the first caller-supplied
            // result and shift every subsequent one.
            mockClient.query.mockResolvedValueOnce({ rows: [], rowCount: 0 });
            for (const result of queryResults) {
                mockClient.query.mockResolvedValueOnce(result);
            }
            // Default fallback for BEGIN/COMMIT/ROLLBACK
            mockClient.query.mockResolvedValue({ rows: [], rowCount: 0 });
            (pool.connect as any).mockResolvedValue(mockClient);
            return mockClient;
        }

        it('should submit a bid when authenticated', async () => {
            const bidData = {
                campaignId: 1,
                amountStroops: 150
            };

            const client = setupClientMock(
                // Auction lookup
                { rows: [{ publisher: otherAddress, floor_price_stroops: '100', status: 'Open' }] },
                // Campaign ownership check
                { rows: [{ advertiser: mockAddress }] },
                // Insert bid
                { rows: [{ id: 'bid-uuid', auction_id: 1, bidder: mockAddress, campaign_id: 1, amount_stroops: 150 }] },
                // Update bid count
                { rows: [] },
                // COMMIT
                { rows: [] },
            );

            const response = await request(app)
                .post('/api/auctions/1/bid')
                .set('Authorization', `Bearer ${token}`)
                .send(bidData);

            expect(response.status).toBe(201);
            expect(response.body.auction_id).toBe(1);
            expect(response.body.amount_stroops).toBe(150);
            expect(client.query).toHaveBeenCalledTimes(6);
            expect(client.release).toHaveBeenCalled();
        });

        it('should return 401 when not authenticated', async () => {
            const response = await request(app)
                .post('/api/auctions/1/bid')
                .send({ campaignId: 1, amountStroops: 150 });

            expect(response.status).toBe(401);
        });

        it('should return 404 when auction does not exist', async () => {
            setupClientMock(
                { rows: [] },
            );

            const response = await request(app)
                .post('/api/auctions/999/bid')
                .set('Authorization', `Bearer ${token}`)
                .send({ campaignId: 1, amountStroops: 150 });

            expect(response.status).toBe(404);
            expect(response.body.error).toBe('Auction not found');
        });

        it('should return 400 when auction is not open', async () => {
            setupClientMock(
                { rows: [{ publisher: otherAddress, floor_price_stroops: '100', status: 'Closed' }] },
            );

            const response = await request(app)
                .post('/api/auctions/1/bid')
                .set('Authorization', `Bearer ${token}`)
                .send({ campaignId: 1, amountStroops: 150 });

            expect(response.status).toBe(400);
            expect(response.body.error).toBe('Auction is not open for bidding');
        });

        it('should return 403 when bidding on own auction', async () => {
            setupClientMock(
                { rows: [{ publisher: mockAddress, floor_price_stroops: '100', status: 'Open' }] },
            );

            const response = await request(app)
                .post('/api/auctions/1/bid')
                .set('Authorization', `Bearer ${token}`)
                .send({ campaignId: 1, amountStroops: 150 });

            expect(response.status).toBe(403);
            expect(response.body.error).toBe('Cannot bid on your own auction');
        });

        it('should return 400 when bid is below floor price', async () => {
            setupClientMock(
                { rows: [{ publisher: otherAddress, floor_price_stroops: '200', status: 'Open' }] },
            );

            const response = await request(app)
                .post('/api/auctions/1/bid')
                .set('Authorization', `Bearer ${token}`)
                .send({ campaignId: 1, amountStroops: 100 });

            expect(response.status).toBe(400);
            expect(response.body.error).toBe('Bid below floor price');
        });

        it('should return 404 when campaign does not exist', async () => {
            setupClientMock(
                { rows: [{ publisher: otherAddress, floor_price_stroops: '100', status: 'Open' }] },
                { rows: [] },
            );

            const response = await request(app)
                .post('/api/auctions/1/bid')
                .set('Authorization', `Bearer ${token}`)
                .send({ campaignId: 999, amountStroops: 150 });

            expect(response.status).toBe(404);
            expect(response.body.error).toBe('Campaign not found');
        });

        it('should return 403 when campaign belongs to another user', async () => {
            setupClientMock(
                { rows: [{ publisher: otherAddress, floor_price_stroops: '100', status: 'Open' }] },
                { rows: [{ advertiser: otherAddress }] },
            );

            const response = await request(app)
                .post('/api/auctions/1/bid')
                .set('Authorization', `Bearer ${token}`)
                .send({ campaignId: 1, amountStroops: 150 });

            expect(response.status).toBe(403);
            expect(response.body.error).toBe('Campaign does not belong to you');
        });
    });
});
