import { describe, it, expect, vi } from 'vitest';
import request from 'supertest';
import app from '../app';

vi.mock('../services/health', () => ({
    runAllChecks: vi.fn().mockResolvedValue({
        database: 'ok',
        redis: 'ok',
        soroban_rpc: 'ok',
        horizon: 'ok',
    })
}));

describe('GET /api/health', () => {
    it('should return 200 and ok status', async () => {
        const response = await request(app).get('/api/health');

        expect(response.status).toBe(200);
        expect(response.body).toHaveProperty('status', 'ok');
        expect(response.body).toHaveProperty('timestamp');
    });
});
