import { Router, Request, Response, NextFunction } from "express";
import { Keypair } from "@stellar/stellar-sdk";
import crypto from "crypto";
import { RateLimiterRedis } from "rate-limiter-flexible";
import type Redis from "ioredis";
import redisClient from "../config/redis";
import { createJwt, decodeJwt, TOKEN_EXPIRY } from "../lib/jwt";
const NONCE_TTL_SECONDS = 300; // 5 minutes
const NONCE_KEY_PREFIX = "nonce:";

async function getChallenge(req: Request, res: Response): Promise<void> {
  const address = req.query.address as string;
  if (!address) {
    res.status(400).json({ error: "Missing address parameter" });
    return;
  }

  try {
    Keypair.fromPublicKey(address);
  } catch {
    res.status(400).json({ error: "Invalid Stellar address" });
    return;
  }

  const nonce = crypto.randomBytes(32).toString("hex");
  await redisClient.set(
    `${NONCE_KEY_PREFIX}${address}`,
    nonce,
    "EX",
    NONCE_TTL_SECONDS,
  );
  res.json({ nonce });
}

export async function verifySignature(
  req: Request,
  res: Response,
): Promise<void> {
  const { address, signature } = req.body;
  if (!address || !signature) {
    res.status(400).json({ error: "Missing address or signature" });
    return;
  }

  const key = `${NONCE_KEY_PREFIX}${address}`;
  const nonce = await redisClient.get(key);
  if (!nonce) {
    res.status(401).json({ error: "No valid challenge found" });
    return;
  }

  try {
    const keypair = Keypair.fromPublicKey(address);
    const valid = keypair.verify(
      Buffer.from(nonce, "hex"),
      Buffer.from(signature, "base64"),
    );
    if (!valid) {
      res.status(401).json({ error: "Invalid signature" });
      return;
    }
  } catch {
    res.status(401).json({ error: "Signature verification failed" });
    return;
  }

  await redisClient.del(key);
  const token = createJwt({ sub: address });
  res.json({ token, expiresIn: TOKEN_EXPIRY });
}

export function requireAuth(
  req: Request,
  res: Response,
  next: NextFunction,
): void {
  const authHeader = req.headers.authorization;
  if (!authHeader?.startsWith("Bearer ")) {
    res.status(401).json({ error: "Missing or invalid Authorization header" });
    return;
  }

  try {
    const payload = decodeJwt(authHeader.slice(7));
    req.stellarAddress = payload.sub;
    next();
  } catch (err: any) {
    res.status(401).json({ error: err.message });
  }
}

export const authRouter = Router();
authRouter.get("/challenge", getChallenge);
authRouter.post("/verify", verifySignature);

/**
 * Redis-backed rate limiters
 */
let ipLimiter: RateLimiterRedis;
let accountLimiter: RateLimiterRedis;
let writeLimiter: RateLimiterRedis;

export function configureRateLimiters(redisClient: Redis): void {
  // Per-IP: 100 requests per minute
  ipLimiter = new RateLimiterRedis({
    storeClient: redisClient,
    keyPrefix: "rl_ip",
    points: 100,
    duration: 60,
  });

  // Per-account: 50 requests per minute
  accountLimiter = new RateLimiterRedis({
    storeClient: redisClient,
    keyPrefix: "rl_acct",
    points: 50,
    duration: 60,
  });

  // Write endpoints: 10 per hour per account
  writeLimiter = new RateLimiterRedis({
    storeClient: redisClient,
    keyPrefix: "rl_write",
    points: 10,
    duration: 3600,
  });
}

export function rateLimit() {
  return async (
    req: Request,
    res: Response,
    next: NextFunction,
  ): Promise<void> => {
    const ip = req.ip || "unknown";

    try {
      await ipLimiter.consume(ip);
    } catch {
      res.status(429).json({ error: "Too many requests" });
      return;
    }

    const address = req.stellarAddress;
    if (address) {
      try {
        await accountLimiter.consume(address);
      } catch {
        res.status(429).json({ error: "Account rate limit exceeded" });
        return;
      }
    }

    next();
  };
}

export function rateLimitWrite() {
  return async (
    req: Request,
    res: Response,
    next: NextFunction,
  ): Promise<void> => {
    const address = req.stellarAddress;
    if (!address) {
      next();
      return;
    }

    try {
      await writeLimiter.consume(address);
      next();
    } catch {
      res
        .status(429)
        .json({ error: "Write rate limit exceeded (10 per hour)" });
      return;
    }
  };
}

/**
 * Middleware: Error handler
 */
export function errorHandler(
  err: Error,
  req: Request,
  res: Response,
  _next: NextFunction,
): void {
  req.log?.error(err, "Internal server error");
  const response: Record<string, string> = { error: "Internal server error" };
  if (process.env.NODE_ENV === "development") {
    response.message = err.message;
  }
  res.status(500).json(response);
}
