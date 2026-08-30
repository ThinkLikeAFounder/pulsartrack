import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createJwt, decodeJwt, TOKEN_EXPIRY } from '../lib/jwt';

// Mock logger to suppress output
vi.mock('../lib/logger', () => ({
  logger: { warn: vi.fn(), error: vi.fn(), info: vi.fn() },
}));

describe('JWT (jwt.ts)', () => {
  it('should create and decode a valid JWT', () => {
    const payload = { sub: 'GABC123' };
    const token = createJwt(payload);
    const decoded = decodeJwt(token);
    expect(decoded.sub).toBe('GABC123');
    expect(decoded.iat).toBeDefined();
    expect(decoded.exp).toBe(decoded.iat + TOKEN_EXPIRY);
  });

  it('should reject a tampered token (invalid signature)', () => {
    const token = createJwt({ sub: 'GABC123' });
    const parts = token.split('.');
    parts[1] = Buffer.from(JSON.stringify({ sub: 'GFAKE', iat: 0, exp: 999999999 })).toString('base64url');
    const tampered = parts.join('.');
    expect(() => decodeJwt(tampered)).toThrow('Invalid token signature');
  });

  it('should reject a malformed token', () => {
    expect(() => decodeJwt('not.a.jwt')).toThrow('Invalid token signature');
    expect(() => decodeJwt('onlytwo')).toThrow('Malformed token');
    expect(() => decodeJwt('')).toThrow('Malformed token');
  });

  it('should reject an expired token', () => {
    const header = Buffer.from(JSON.stringify({ alg: 'HS256', typ: 'JWT' })).toString('base64url');
    const body = Buffer.from(JSON.stringify({ sub: 'GABC', iat: 0, exp: 1 })).toString('base64url');
    const crypto = require('crypto');
    const sig = crypto.createHmac('sha256', process.env.JWT_SECRET || 'test').update(`${header}.${body}`).digest('base64url');
    const expiredToken = `${header}.${body}.${sig}`;
    expect(() => decodeJwt(expiredToken)).toThrow('Token expired');
  });
});

describe('requireAuth middleware', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should reject request with no Authorization header', async () => {
    const { default: app } = await import('../app');
    const request = (await import('supertest')).default;
    // /api/contracts uses requireAuth
    const res = await request(app).get('/api/contracts');
    expect(res.status).toBe(401);
  });

  it('should reject request with invalid token', async () => {
    const { default: app } = await import('../app');
    const request = (await import('supertest')).default;
    const res = await request(app)
      .get('/api/contracts')
      .set('Authorization', 'Bearer invalid.token.here');
    expect(res.status).toBe(401);
  });

  it('should reject request with expired token', async () => {
    const { default: app } = await import('../app');
    const request = (await import('supertest')).default;
    const header = Buffer.from(JSON.stringify({ alg: 'HS256', typ: 'JWT' })).toString('base64url');
    const body = Buffer.from(JSON.stringify({ sub: 'GABC', iat: 0, exp: 1 })).toString('base64url');
    const crypto = require('crypto');
    const jwtSecret = process.env.JWT_SECRET || 'test';
    const sig = crypto.createHmac('sha256', jwtSecret).update(`${header}.${body}`).digest('base64url');
    const expiredToken = `${header}.${body}.${sig}`;
    const res = await request(app)
      .get('/api/contracts')
      .set('Authorization', `Bearer ${expiredToken}`);
    expect(res.status).toBe(401);
    expect(res.body.error).toMatch(/expired/i);
  });

  it('should accept request with valid token', async () => {
    const { default: app } = await import('../app');
    const request = (await import('supertest')).default;
    const token = createJwt({ sub: 'GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N' });
    const res = await request(app)
      .get('/api/contracts')
      .set('Authorization', `Bearer ${token}`);
    expect(res.status).toBe(200);
    expect(res.body.contracts).toBeDefined();
  });
});
