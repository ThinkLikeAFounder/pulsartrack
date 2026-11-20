import { describe, it, expect, vi, beforeEach, beforeAll } from 'vitest';
import { createHmac } from 'crypto';
import { createJwt, decodeJwt, InvalidTokenError, TOKEN_EXPIRY } from '../lib/jwt';

// Mock logger to suppress output
vi.mock('../lib/logger', () => ({
  logger: { warn: vi.fn(), error: vi.fn(), info: vi.fn() },
}));

// Mock Redis to avoid real connections
vi.mock('../config/redis', () => ({
  default: {
    get: vi.fn().mockResolvedValue(null),
    set: vi.fn().mockResolvedValue('OK'),
    del: vi.fn().mockResolvedValue(1),
    incr: vi.fn().mockResolvedValue(1),
    expire: vi.fn().mockResolvedValue(1),
    disconnect: vi.fn().mockResolvedValue(undefined),
  },
}));

// Mock database to avoid real connections
vi.mock('../config/database', () => ({
  query: vi.fn().mockResolvedValue({ rows: [] }),
  pool: {
    query: vi.fn().mockResolvedValue({ rows: [] }),
    end: vi.fn().mockResolvedValue(undefined),
  },
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
    expect(() => decodeJwt(tampered)).toThrow(InvalidTokenError);
    expect(() => decodeJwt(tampered)).toThrow('Invalid token signature');
  });

  describe('should reject malformed tokens with InvalidTokenError (not SyntaxError)', () => {
    it('should reject token with wrong segment count', () => {
      expect(() => decodeJwt('onlytwosegments.here')).toThrow(InvalidTokenError);
      expect(() => decodeJwt('')).toThrow(InvalidTokenError);
    });

    it('should reject token with invalid base64', () => {
      expect(() => decodeJwt('not.a.jwt')).toThrow(InvalidTokenError);
      expect(() => decodeJwt('not.a.jwt')).toThrow('Invalid token signature');
    });

    it('should reject validly signed token with non-JSON header', () => {
      const header = Buffer.from('not-json').toString('base64url');
      const body = Buffer.from(JSON.stringify({ sub: 'GABC', iat: 0, exp: 999999999999 })).toString('base64url');
      const sig = createHmac('sha256', process.env.JWT_SECRET || 'test').update(`${header}.${body}`).digest('base64url');
      const malformedToken = `${header}.${body}.${sig}`;
      expect(() => decodeJwt(malformedToken)).toThrow(InvalidTokenError);
      expect(() => decodeJwt(malformedToken)).toThrow('Invalid token header');
    });

    it('should reject validly signed token with non-JSON payload', () => {
      const header = Buffer.from(JSON.stringify({ alg: 'HS256', typ: 'JWT' })).toString('base64url');
      const body = Buffer.from('not-json').toString('base64url');
      const sig = createHmac('sha256', process.env.JWT_SECRET || 'test').update(`${header}.${body}`).digest('base64url');
      const malformedToken = `${header}.${body}.${sig}`;
      expect(() => decodeJwt(malformedToken)).toThrow(InvalidTokenError);
      expect(() => decodeJwt(malformedToken)).toThrow('Invalid token payload');
    });

    it('should reject token with wrong algorithm', () => {
      const header = Buffer.from(JSON.stringify({ alg: 'HS512', typ: 'JWT' })).toString('base64url');
      const body = Buffer.from(JSON.stringify({ sub: 'GABC', iat: 0, exp: 999999999999 })).toString('base64url');
      const sig = createHmac('sha256', process.env.JWT_SECRET || 'test').update(`${header}.${body}`).digest('base64url');
      const wrongAlgToken = `${header}.${body}.${sig}`;
      expect(() => decodeJwt(wrongAlgToken)).toThrow(InvalidTokenError);
      expect(() => decodeJwt(wrongAlgToken)).toThrow('Invalid token header');
    });
  });

  it('should reject an expired token', () => {
    const header = Buffer.from(JSON.stringify({ alg: 'HS256', typ: 'JWT' })).toString('base64url');
    const body = Buffer.from(JSON.stringify({ sub: 'GABC', iat: 0, exp: 1 })).toString('base64url');
    const sig = createHmac('sha256', process.env.JWT_SECRET || 'test').update(`${header}.${body}`).digest('base64url');
    const expiredToken = `${header}.${body}.${sig}`;
    expect(() => decodeJwt(expiredToken)).toThrow(InvalidTokenError);
    expect(() => decodeJwt(expiredToken)).toThrow('Token expired');
  });
});

describe('requireAuth middleware', () => {
  let app: any;
  let request: any;

  // Import app once before all tests (not in each test)
  beforeAll(async () => {
    const appModule = await import('../app');
    app = appModule.default;
    request = (await import('supertest')).default;
  });

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should reject request with no Authorization header', async () => {
    // /api/contracts uses requireAuth
    const res = await request(app).get('/api/contracts');
    expect(res.status).toBe(401);
  });

  it('should reject request with invalid token', async () => {
    const res = await request(app)
      .get('/api/contracts')
      .set('Authorization', 'Bearer invalid.token.here');
    expect(res.status).toBe(401);
  });

  it('should reject request with expired token', async () => {
    const header = Buffer.from(JSON.stringify({ alg: 'HS256', typ: 'JWT' })).toString('base64url');
    const body = Buffer.from(JSON.stringify({ sub: 'GABC', iat: 0, exp: 1 })).toString('base64url');
    const jwtSecret = process.env.JWT_SECRET || 'test';
    const sig = createHmac('sha256', jwtSecret).update(`${header}.${body}`).digest('base64url');
    const expiredToken = `${header}.${body}.${sig}`;
    const res = await request(app)
      .get('/api/contracts')
      .set('Authorization', `Bearer ${expiredToken}`);
    expect(res.status).toBe(401);
    expect(res.body.error).toMatch(/expired/i);
  });

  it('should accept request with valid token', async () => {
    const token = createJwt({ sub: 'GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N' });
    const res = await request(app)
      .get('/api/contracts')
      .set('Authorization', `Bearer ${token}`);
    expect(res.status).toBe(200);
    expect(res.body.contracts).toBeDefined();
  });

  it('should enforce per-account rate limiting and return 429 when points are exhausted', async () => {
    const { accountLimiter } = await import('../middleware/auth');
    const address = 'GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N';
    const token = createJwt({ sub: address });

    const consumeSpy = vi.spyOn(accountLimiter, 'consume').mockRejectedValueOnce(new Error('Rate limit exceeded'));

    try {
      const res = await request(app)
        .get('/api/contracts')
        .set('Authorization', `Bearer ${token}`);

      expect(res.status).toBe(429);
      expect(res.body).toEqual({ error: 'Account rate limit exceeded' });
      expect(consumeSpy).toHaveBeenCalledWith(address);
    } finally {
      consumeSpy.mockRestore();
    }
  });

  it('should return 429 when authenticated requests exceed the 50 req/min per-account limit', async () => {
    const { accountLimiter } = await import('../middleware/auth');
    const address = 'GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N';
    const token = createJwt({ sub: address });

    let callCount = 0;
    const consumeSpy = vi.spyOn(accountLimiter, 'consume').mockImplementation(async () => {
      callCount++;
      if (callCount > 50) {
        throw new Error('Rate limit exceeded');
      }
      return {} as any;
    });

    try {
      // Make 50 allowed requests
      for (let i = 0; i < 50; i++) {
        const okRes = await request(app)
          .get('/api/contracts')
          .set('Authorization', `Bearer ${token}`);
        expect(okRes.status).toBe(200);
      }

      // 51st request exceeds the per-account rate limit
      const exceededRes = await request(app)
        .get('/api/contracts')
        .set('Authorization', `Bearer ${token}`);
      expect(exceededRes.status).toBe(429);
      expect(exceededRes.body).toEqual({ error: 'Account rate limit exceeded' });
      expect(consumeSpy).toHaveBeenCalledWith(address);
    } finally {
      consumeSpy.mockRestore();
    }
  });
});

describe('rateLimitAccount middleware', () => {
  it('should call next() if req.stellarAddress is not set', async () => {
    const { rateLimitAccount } = await import('../middleware/auth');
    const mockReq = {} as any;
    const mockRes = { status: vi.fn().mockReturnThis(), json: vi.fn() } as any;
    const next = vi.fn();

    const middleware = rateLimitAccount();
    await middleware(mockReq, mockRes, next);

    expect(next).toHaveBeenCalledTimes(1);
    expect(mockRes.status).not.toHaveBeenCalled();
  });

  it('should consume account rate limit points when req.stellarAddress is present', async () => {
    const { rateLimitAccount, accountLimiter } = await import('../middleware/auth');
    const consumeSpy = vi.spyOn(accountLimiter, 'consume').mockResolvedValueOnce({} as any);
    try {
      const mockReq = { stellarAddress: 'GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N' } as any;
      const mockRes = { status: vi.fn().mockReturnThis(), json: vi.fn() } as any;
      const next = vi.fn();

      const middleware = rateLimitAccount();
      await middleware(mockReq, mockRes, next);

      expect(consumeSpy).toHaveBeenCalledWith('GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N');
      expect(next).toHaveBeenCalledTimes(1);
    } finally {
      consumeSpy.mockRestore();
    }
  });

  it('should respond with 429 when accountLimiter rejects', async () => {
    const { rateLimitAccount, accountLimiter } = await import('../middleware/auth');
    const consumeSpy = vi.spyOn(accountLimiter, 'consume').mockRejectedValueOnce(new Error('Rate limit exceeded'));
    try {
      const mockReq = { stellarAddress: 'GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N' } as any;
      const mockRes = { status: vi.fn().mockReturnThis(), json: vi.fn() } as any;
      const next = vi.fn();

      const middleware = rateLimitAccount();
      await middleware(mockReq, mockRes, next);

      expect(next).not.toHaveBeenCalled();
      expect(mockRes.status).toHaveBeenCalledWith(429);
      expect(mockRes.json).toHaveBeenCalledWith({ error: 'Account rate limit exceeded' });
    } finally {
      consumeSpy.mockRestore();
    }
  });
});

describe('getAuthedAddress helper', () => {
  it('should return address when req.stellarAddress is set', async () => {
    const { getAuthedAddress } = await import('../middleware/auth');
    const mockReq = { stellarAddress: 'GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N' } as any;
    const mockRes = { status: vi.fn().mockReturnThis(), json: vi.fn() } as any;
    
    const address = getAuthedAddress(mockReq, mockRes);
    
    expect(address).toBe('GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N');
    expect(mockRes.status).not.toHaveBeenCalled();
    expect(mockRes.json).not.toHaveBeenCalled();
  });

  it('should return null and send 401 when req.stellarAddress is missing', async () => {
    const { getAuthedAddress } = await import('../middleware/auth');
    const mockReq = {} as any;
    const mockRes = { status: vi.fn().mockReturnThis(), json: vi.fn() } as any;
    
    const address = getAuthedAddress(mockReq, mockRes);
    
    expect(address).toBeNull();
    expect(mockRes.status).toHaveBeenCalledWith(401);
    expect(mockRes.json).toHaveBeenCalledWith({ error: 'Authentication required' });
  });

  it('should return null and send 401 when req.stellarAddress is undefined', async () => {
    const { getAuthedAddress } = await import('../middleware/auth');
    const mockReq = { stellarAddress: undefined } as any;
    const mockRes = { status: vi.fn().mockReturnThis(), json: vi.fn() } as any;
    
    const address = getAuthedAddress(mockReq, mockRes);
    
    expect(address).toBeNull();
    expect(mockRes.status).toHaveBeenCalledWith(401);
    expect(mockRes.json).toHaveBeenCalledWith({ error: 'Authentication required' });
  });
});

describe('Live auth routes (/api/auth)', () => {
  let app: any;
  let request: any;
  let redisStore: Map<string, string>;

  beforeAll(async () => {
    const appModule = await import('../app');
    app = appModule.default;
    request = (await import('supertest')).default;
  });

  beforeEach(async () => {
    vi.clearAllMocks();
    redisStore = new Map();
    const redisModule = await import('../config/redis');
    const redisClient = redisModule.default;
    vi.spyOn(redisClient, 'set').mockImplementation(async (key: any, val: any) => {
      redisStore.set(String(key), String(val));
      return 'OK' as any;
    });
    vi.spyOn(redisClient, 'get').mockImplementation(async (key: any) => {
      return (redisStore.get(String(key)) ?? null) as any;
    });
    vi.spyOn(redisClient, 'del').mockImplementation(async (key: any) => {
      redisStore.delete(String(key));
      return 1 as any;
    });
  });

  it('should complete full authentication flow through the live app and return usable JWT', async () => {
    const { Keypair } = await import('@stellar/stellar-sdk');
    const keypair = Keypair.random();
    const address = keypair.publicKey();

    // 1. Hit GET /api/auth/challenge through live app
    const challengeRes = await request(app)
      .get(`/api/auth/challenge?address=${address}`);
    expect(challengeRes.status).toBe(200);
    expect(challengeRes.body).toHaveProperty('nonce');
    const { nonce } = challengeRes.body;
    expect(typeof nonce).toBe('string');
    expect(nonce.length).toBe(64);

    // 2. Sign nonce using Stellar keypair
    const signatureBytes = keypair.sign(Buffer.from(nonce, 'hex'));
    const signature = Buffer.from(signatureBytes).toString('base64');

    // 3. Hit POST /api/auth/verify through live app
    const verifyRes = await request(app)
      .post('/api/auth/verify')
      .send({ address, signature });
    expect(verifyRes.status).toBe(200);
    expect(verifyRes.body).toHaveProperty('token');
    expect(verifyRes.body).toHaveProperty('expiresIn');
    const { token } = verifyRes.body;
    expect(typeof token).toBe('string');

    // 4. Confirm the returned token is usable against a protected endpoint
    const protectedRes = await request(app)
      .get('/api/contracts')
      .set('Authorization', `Bearer ${token}`);
    expect(protectedRes.status).toBe(200);
    expect(protectedRes.body).toHaveProperty('contracts');
  });

  it('should return 400 when requesting challenge with invalid Stellar address', async () => {
    const res = await request(app).get('/api/auth/challenge?address=invalid_address');
    expect(res.status).toBe(400);
    expect(res.body).toHaveProperty('error', 'Invalid Stellar address');
  });

  it('should return 400 when address is missing in challenge request', async () => {
    const res = await request(app).get('/api/auth/challenge');
    expect(res.status).toBe(400);
    expect(res.body).toHaveProperty('error', 'Missing address parameter');
  });

  it('should return 401 when verifying with an invalid signature', async () => {
    const { Keypair } = await import('@stellar/stellar-sdk');
    const keypair = Keypair.random();
    const address = keypair.publicKey();

    // Store a challenge in redis
    redisStore.set(`nonce:${address}`, 'abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234');

    const invalidSignature = Buffer.from('invalid-sig').toString('base64');
    const res = await request(app)
      .post('/api/auth/verify')
      .send({ address, signature: invalidSignature });
    expect(res.status).toBe(401);
  });
});

