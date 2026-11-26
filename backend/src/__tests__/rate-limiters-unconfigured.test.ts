import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { NextFunction, Request, Response } from 'express';

vi.mock('../lib/logger', () => ({
  logger: { warn: vi.fn(), error: vi.fn(), info: vi.fn() },
}));

vi.mock('../config/redis', () => ({
  default: {},
}));

function mockRes() {
  const res = {} as Response;
  res.status = vi.fn().mockReturnValue(res);
  res.json = vi.fn().mockReturnValue(res);
  return res;
}

const req = { ip: '127.0.0.1', stellarAddress: 'GABC123' } as unknown as Request;

describe('rate limiters before configureRateLimiters() runs', () => {
  beforeEach(() => {
    // Fresh module instance so the limiters are guaranteed to be undefined
    vi.resetModules();
  });

  it.each(['rateLimit', 'rateLimitAccount', 'rateLimitWrite'] as const)(
    '%s skips rate limiting instead of responding with 429',
    async (name) => {
      const mod = await import('../middleware/auth');
      const res = mockRes();
      const next = vi.fn() as unknown as NextFunction;

      await mod[name]()(req, res, next);

      expect(next).toHaveBeenCalledTimes(1);
      expect(res.status).not.toHaveBeenCalled();
    },
  );
});
