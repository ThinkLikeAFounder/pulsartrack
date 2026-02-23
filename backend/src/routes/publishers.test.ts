import { describe, it, expect, vi } from 'vitest';
import request from 'supertest';
import app from '../app';
import pool from '../config/database';
import { generateTestToken } from '../test-utils';

describe('Publisher Routes', () => {
    const mockAddress = 'GD7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7';
    const token = generateTestToken(mockAddress);

    describe('GET /api/publishers/leaderboard', () => {
        it('should return publisher leaderboard', async () => {
            (pool.query as any).mockResolvedValue({
                rows: [
                    {
                        address: mockAddress,
                        display_name: 'Top Pub',
                        tier: 'Gold',
                        reputation_score: 900,
                        impressions_served: '10000',
                        earnings_stroops: '500000000',
                        last_activity: new Date()
                    }
                ]
            });

            const response = await request(app).get('/api/publishers/leaderboard');

            expect(response.status).toBe(200);
            expect(response.body).toHaveProperty('publishers');
            expect(response.body.publishers[0].displayName).toBe('Top Pub');
        });
    });

    describe('POST /api/publishers/register', () => {
        it('should register a publisher when authenticated', async () => {
            const pubData = {
                displayName: 'New Publisher',
                website: 'https://newpub.com'
            };

            (pool.query as any).mockResolvedValue({
                rows: [{
                    id: 'pub-uuid',
                    address: mockAddress,
                    display_name: pubData.displayName,
                    website: pubData.website
                }]
            });

            const response = await request(app)
                .post('/api/publishers/register')
                .set('Authorization', `Bearer ${token}`)
                .send(pubData);

            expect(response.status).toBe(201);
            expect(response.body.display_name).toBe(pubData.displayName);
        });

        it('should return 401 when not authenticated', async () => {
            const response = await request(app)
                .post('/api/publishers/register')
                .send({ displayName: 'Anon' });

            expect(response.status).toBe(401);
        });
    });
});
