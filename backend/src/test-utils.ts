import crypto from 'crypto';

const JWT_SECRET = process.env.JWT_SECRET || 'test-secret-key-12345';

/**
 * Generates a mock JWT for testing
 */
export function generateTestToken(address: string): string {
    const header = Buffer.from(JSON.stringify({ alg: 'HS256', typ: 'JWT' })).toString('base64url');
    const now = Math.floor(Date.now() / 1000);
    const payload = Buffer.from(JSON.stringify({
        sub: address,
        iat: now,
        exp: now + 3600
    })).toString('base64url');

    const sig = crypto.createHmac('sha256', JWT_SECRET).update(`${header}.${payload}`).digest('base64url');
    return `${header}.${payload}.${sig}`;
}

/**
 * Mock data generators
 */
export const mockData = {
    campaign: (id = '1') => ({
        id: `uuid-${id}`,
        campaignId: BigInt(id),
        advertiser: 'GB7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7',
        title: 'Test Campaign',
        contentId: 'cid-123',
        budgetStroops: BigInt(10000000),
        dailyBudgetStroops: BigInt(1000000),
        spentStroops: BigInt(0),
        impressions: BigInt(0),
        clicks: BigInt(0),
        status: 'Active',
        createdAt: new Date(),
    }),
    publisher: (address = 'GD7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7') => ({
        id: 'uuid-pub',
        address,
        displayName: 'Test Publisher',
        website: 'https://example.com',
        status: 'Active',
        tier: 'Bronze',
        reputationScore: 500,
        joinedAt: new Date(),
    }),
    auction: (id = '1') => ({
        id: `uuid-auc-${id}`,
        auctionId: BigInt(id),
        publisher: 'GD7V7Z5K64I6U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7U6I7',
        impressionSlot: 'banner-top',
        floorPriceStroops: BigInt(100),
        reservePriceStroops: BigInt(200),
        status: 'Open',
        startTime: new Date(),
        endTime: new Date(Date.now() + 3600000),
    }),
};
