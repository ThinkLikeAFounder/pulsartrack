import { describe, it, expect, vi, beforeEach } from 'vitest';
import request from 'supertest';
import app from '../app';
import prisma from '../db/prisma';
import { generateTestToken } from '../test-utils';
import { callReadOnly } from '../services/soroban-client';
import { CONTRACT_IDS } from '../config/stellar';

describe('Publisher Routes', () => {
    const mockAddress = 'GD7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7';
    const token = generateTestToken(mockAddress);

    beforeEach(() => {
        vi.resetAllMocks();
    });

    describe('GET /api/publishers/leaderboard', () => {
        it('should return publisher leaderboard', async () => {
            (prisma.publisher.findMany as any).mockResolvedValue([
                {
                    id: 'pub-uuid',
                    address: mockAddress,
                    displayName: 'Top Pub',
                    tier: 'Gold',
                    reputationScore: 900,
                    impressionsServed: BigInt(10000),
                    earningsStroops: BigInt(500000000),
                    lastActivity: new Date(),
                    website: null,
                    status: 'Verified',
                    createdAt: new Date(),
                    updatedAt: new Date(),
                }
            ]);

            const response = await request(app).get('/api/publishers/leaderboard');

            expect(response.status).toBe(200);
            expect(response.body).toHaveProperty('publishers');
            expect(response.body.publishers[0].displayName).toBe('Top Pub');
        });

        it('refreshes the on-chain reputation score for every publisher', async () => {
            const secondAddress = 'GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N';
            CONTRACT_IDS.PUBLISHER_REPUTATION = 'CREPUTATION';
            (prisma.publisher.findMany as any).mockResolvedValue([
                {
                    id: 'pub-uuid-1', address: mockAddress, displayName: 'Top Pub', tier: 'Gold',
                    reputationScore: 900, impressionsServed: BigInt(10000), earningsStroops: BigInt(500000000),
                    lastActivity: new Date(), website: null, status: 'Verified', createdAt: new Date(), updatedAt: new Date(),
                },
                {
                    id: 'pub-uuid-2', address: secondAddress, displayName: 'Second Pub', tier: 'Silver',
                    reputationScore: 800, impressionsServed: BigInt(5000), earningsStroops: BigInt(250000000),
                    lastActivity: new Date(), website: null, status: 'Verified', createdAt: new Date(), updatedAt: new Date(),
                },
            ]);
            (callReadOnly as any).mockResolvedValueOnce(950).mockResolvedValueOnce(850);

            const response = await request(app).get('/api/publishers/leaderboard');

            expect(response.status).toBe(200);
            expect(callReadOnly).toHaveBeenCalledTimes(2);
            expect(response.body.publishers.map((publisher: { reputationScore: number }) => publisher.reputationScore))
                .toEqual([950, 850]);
            CONTRACT_IDS.PUBLISHER_REPUTATION = '';
        });
    });

    describe('POST /api/publishers/register', () => {
        it('should register a publisher when authenticated', async () => {
            const pubData = {
                displayName: 'New Publisher',
                website: 'https://newpub.com'
            };

            // Mock findByAddress to return null (publisher doesn't exist)
            (prisma.publisher.findUnique as any).mockResolvedValue(null);

            // Mock create to return new publisher
            (prisma.publisher.create as any).mockResolvedValue({
                id: 'pub-uuid',
                address: mockAddress,
                displayName: pubData.displayName,
                website: pubData.website,
                tier: 'Bronze',
                status: 'Pending',
                reputationScore: 0,
                impressionsServed: BigInt(0),
                earningsStroops: BigInt(0),
                lastActivity: new Date(),
                createdAt: new Date(),
                updatedAt: new Date(),
            });

            const response = await request(app)
                .post('/api/publishers/register')
                .set('Authorization', `Bearer ${token}`)
                .send(pubData);

            expect(response.status).toBe(201);
            expect(response.body.displayName).toBe(pubData.displayName);
        });

        it('should return 401 when not authenticated', async () => {
            const response = await request(app)
                .post('/api/publishers/register')
                .send({ displayName: 'Anon' });

            expect(response.status).toBe(401);
        });

        it('should return 409 when publisher already registered', async () => {
            // Mock findByAddress to return an existing publisher
            (prisma.publisher.findUnique as any).mockResolvedValue({
                id: 'existing-pub-uuid',
                address: mockAddress,
                displayName: 'Existing Publisher',
                website: 'https://existing.com',
                tier: 'Gold',
                status: 'Verified',
                reputationScore: 800,
                impressionsServed: BigInt(5000),
                earningsStroops: BigInt(200000000),
                lastActivity: new Date(),
                createdAt: new Date(),
                updatedAt: new Date(),
            });

            const response = await request(app)
                .post('/api/publishers/register')
                .set('Authorization', `Bearer ${token}`)
                .send({ displayName: 'Duplicate', website: 'https://dup.com' });

            expect(response.status).toBe(409);
            expect(response.body.error).toBe('Publisher already registered');
        });
    });
});
