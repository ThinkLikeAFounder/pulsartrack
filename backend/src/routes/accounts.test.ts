import { describe, it, expect, vi } from 'vitest';
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
    // Valid 56-character Stellar address
    const mockAddress = 'GA5W6GSR6G2CXP747U7S6ZPH5EALQY57V22K6YJSP2XYG47YJ3PGLRTI';

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
        expect(response.body).toHaveProperty('error', errorMsg);
    });
});
