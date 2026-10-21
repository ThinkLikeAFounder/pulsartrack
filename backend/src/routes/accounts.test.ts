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

    it.each(['0', '-5'])('clamps limit=%s to one transaction', async (limit) => {
        (horizon.getAccountTransactions as any).mockResolvedValue({ records: [] });

        const response = await request(app)
            .get(`/api/account/${mockAddress}/transactions?limit=${limit}`);

        expect(response.status).toBe(200);
        expect(horizon.getAccountTransactions).toHaveBeenCalledWith(mockAddress, 1, undefined, 'desc');
    });

    it('should return 400 for malformed address when requesting account details', async () => {
        const response = await request(app).get('/api/account/not-a-valid-stellar-address');

        expect(response.status).toBe(400);
        expect(response.body).toHaveProperty('error', 'Validation failed');
        expect(response.body.details).toEqual(
            expect.arrayContaining([
                expect.objectContaining({
                    field: 'address',
                    message: 'address must be a valid Stellar public key',
                }),
            ]),
        );
        expect(horizon.getAccountDetails).not.toHaveBeenCalled();
    });

    it('should return 400 for malformed address when requesting transactions', async () => {
        const response = await request(app).get('/api/account/not-a-valid-stellar-address/transactions');

        expect(response.status).toBe(400);
        expect(response.body).toHaveProperty('error', 'Validation failed');
        expect(response.body.details).toEqual(
            expect.arrayContaining([
                expect.objectContaining({
                    field: 'address',
                    message: 'address must be a valid Stellar public key',
                }),
            ]),
        );
        expect(horizon.getAccountTransactions).not.toHaveBeenCalled();
    });

    it('should return 200 and paginated transactions for valid address', async () => {
        const mockTxs = [
            { id: 'tx1', paging_token: '1000' },
            { id: 'tx2', paging_token: '2000' },
        ];
        (horizon.getAccountTransactions as any).mockResolvedValue({ records: mockTxs });

        const response = await request(app)
            .get(`/api/account/${mockAddress}/transactions?limit=2&order=asc&cursor=500`);

        expect(response.status).toBe(200);
        expect(horizon.getAccountTransactions).toHaveBeenCalledWith(mockAddress, 2, '500', 'asc');
        expect(response.body).toEqual({
            transactions: mockTxs,
            count: 2,
            cursor: {
                current: '500',
                next: '2000',
            },
            order: 'asc',
        });
    });
});

