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
        const EMPTY = { rows: [], rowCount: 0 };

        /**
         * Queues mocked responses in the exact order the bid handler issues them:
         *
         *   1. BEGIN
         *   2. SELECT ... FROM auctions ... FOR UPDATE   <- queryResults[0]
         *   3. SELECT advertiser FROM campaigns          <- queryResults[1]
         *   4. INSERT INTO bids ... RETURNING *          <- queryResults[2]
         *   5. UPDATE auctions SET bid_count = ...       <- queryResults[3]
         *   6. COMMIT
         *
         * Callers pass only the data queries; BEGIN is queued here and
         * COMMIT/ROLLBACK fall through to the default empty result.
         */
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
            // Default fallback for COMMIT/ROLLBACK
            mockClient.query.mockResolvedValue(EMPTY);
            (pool.connect as any).mockResolvedValue(mockClient);
            return mockClient;
        }

        it('should submit a bid when authenticated', async () => {
            const bidData = {
                campaignId: 1,
                amountStroops: 150
            };

            const mockClient = {
                query: vi.fn()
                    .mockResolvedValueOnce({}) // BEGIN
                    .mockResolvedValueOnce({   // INSERT
                        rows: [{
                            id: 'bid-uuid',
                            auction_id: 1,
                            bidder: mockAddress,
                            campaign_id: bidData.campaignId,
                            amount_stroops: bidData.amountStroops
                        }]
                    })
                    .mockResolvedValueOnce({}) // UPDATE
                    .mockResolvedValueOnce({}), // COMMIT
                release: vi.fn()
            };
            vi.spyOn(pool, 'connect').mockResolvedValueOnce(mockClient as any);
            const client = setupClientMock(
                // Auction lookup (SELECT ... FOR UPDATE)
                { rows: [{ publisher: otherAddress, floor_price_stroops: '100', status: 'Open' }] },
                // Campaign ownership check
                { rows: [{ advertiser: mockAddress }] },
                // Insert bid
                { rows: [{ id: 'bid-uuid', auction_id: 1, bidder: mockAddress, campaign_id: 1, amount_stroops: 150 }] },
                // Update bid count
                { rows: [] },
            );

            const response = await request(app)
                .post('/api/auctions/1/bid')
                .set('Authorization', `Bearer ${token}`)
                .send(bidData);

            expect(response.status).toBe(201);
            expect(response.body.auction_id).toBe(1);
            expect(response.body.amount_stroops).toBe(150);
            // BEGIN, auction lock, campaign check, insert, bid_count update, COMMIT
            expect(client.query).toHaveBeenCalledTimes(6);
            const calls = client.query.mock.calls.map((c: any[]) => c[0]);
            expect(calls[0]).toBe('BEGIN');
            expect(calls[1]).toContain('FOR UPDATE');
            expect(calls[1]).toContain('FROM auctions');
            expect(calls[2]).toContain('FROM campaigns');
            expect(calls[5]).toBe('COMMIT');
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

        describe('TOCTOU: concurrent auction close vs. bid (#794)', () => {
            it('locks the auction row inside the transaction before reading its status', async () => {
                const client = setupClientMock(
                    { rows: [{ publisher: otherAddress, floor_price_stroops: '100', status: 'Open' }] },
                    { rows: [{ advertiser: mockAddress }] },
                    { rows: [{ id: 'bid-uuid', auction_id: 1, bidder: mockAddress, campaign_id: 1, amount_stroops: 150 }] },
                    { rows: [] },
                );

                const response = await request(app)
                    .post('/api/auctions/1/bid')
                    .set('Authorization', `Bearer ${token}`)
                    .send({ campaignId: 1, amountStroops: 150 });

                expect(response.status).toBe(201);

                const calls = client.query.mock.calls.map((c: any[]) => c[0]);
                const beginIndex = calls.indexOf('BEGIN');
                const lockIndex = calls.findIndex(
                    (sql: string) => typeof sql === 'string' && sql.includes('FROM auctions') && sql.includes('FOR UPDATE'),
                );
                const commitIndex = calls.indexOf('COMMIT');

                // The auction must be read under a row lock, and that lock must be
                // acquired after BEGIN and released only at COMMIT — otherwise a
                // concurrent close could land between the status read and the insert.
                expect(beginIndex).toBe(0);
                expect(lockIndex).toBeGreaterThan(beginIndex);
                expect(commitIndex).toBeGreaterThan(lockIndex);
            });

            it('rejects a bid when the concurrent close won the race and the locked row reads Closed', async () => {
                const client = setupClientMock(
                    // The competing transaction committed the close first, so the
                    // locked read returns the post-close state.
                    { rows: [{ publisher: otherAddress, floor_price_stroops: '100', status: 'Closed' }] },
                );

                const response = await request(app)
                    .post('/api/auctions/1/bid')
                    .set('Authorization', `Bearer ${token}`)
                    .send({ campaignId: 1, amountStroops: 150 });

                expect(response.status).toBe(400);
                expect(response.body.error).toBe('Auction is not open for bidding');

                const calls = client.query.mock.calls.map((c: any[]) => c[0]);
                expect(calls[0]).toBe('BEGIN');
                expect(calls[calls.length - 1]).toBe('ROLLBACK');
                // No bid may be inserted once the locked read shows a closed auction.
                expect(calls.some((sql: string) => typeof sql === 'string' && sql.includes('INSERT INTO bids'))).toBe(false);
                expect(client.release).toHaveBeenCalled();
            });
        });
    });
});
