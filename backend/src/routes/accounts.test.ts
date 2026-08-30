import { describe, it, expect, vi, beforeEach } from 'vitest';
import request from 'supertest';
import app from '../app';
import * as horizon from '../services/horizon';

// Mock horizon service
vi.mock('../services/horizon', () => ({
    getAccountDetails: vi.fn(),
    getAccountTransactions: vi.fn(),
    getFeeStats: vi.fn(),
}));

describe('GET /api/account/:address', () => {
    // Valid Stellar address (verified via Keypair.fromPublicKey)
    const mockAddress = 'GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N';

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should return 200 and account data for valid address', async () => {
        const mockAccount = {
            address: mockAddress,
            xlmBalance: 100.0,
            balances: [{ asset_type: 'native', balance: '100.0' }],
        };
        (horizon.getAccountDetails as any).mockResolvedValue(mockAccount);

        const response = await request(app).get(`/api/account/${mockAddress}`);

        expect(response.status).toBe(200);
        expect(response.body).toEqual(mockAccount);
    });

    it('should return 404 for non-existent account', async () => {
        (horizon.getAccountDetails as any).mockResolvedValue(null);

        const response = await request(app).get(`/api/account/${mockAddress}`);

        expect(response.status).toBe(404);
        expect(response.body).toHaveProperty('error', 'Account not found or not funded');
    });

    it('should return 500 if horizon service fails', async () => {
        const errorMsg = 'Horizon service unavailable';
        (horizon.getAccountDetails as any).mockRejectedValue(new Error(errorMsg));

        const response = await request(app).get(`/api/account/${mockAddress}`);

        expect(response.status).toBe(500);
        // The error handler in auth.ts returns { error: 'Internal server error', message: err.message }
        // but the route handler itself returns { error: err.message }
        expect(response.body).toHaveProperty('error', 'Failed to fetch account details');
        expect(response.body).toHaveProperty('error', 'Failed to fetch account details');
        expect(response.body).not.toHaveProperty('details');
    });
});
