import { checkDbConnection } from "../config/database";
import redisClient from "../config/redis";
import { getHorizonServer } from "../config/stellar";
import { getServer } from "./soroban-client";
import { logger } from "../lib/logger";

export interface HealthCheckResult {
  database: "ok" | "error";
  redis: "ok" | "error";
  soroban_rpc: "ok" | "error";
  horizon: "ok" | "error";
}

export async function checkDatabase(): Promise<"ok" | "error"> {
  try {
    const isConnected = await checkDbConnection();
    return isConnected ? "ok" : "error";
  } catch {
    return "error";
  }
}

export async function checkRedis(): Promise<"ok" | "error"> {
  try {
    const res = await redisClient.ping();
    return res === "PONG" ? "ok" : "error";
  } catch {
    return "error";
  }
}

export async function checkHorizon(): Promise<"ok" | "error"> {
  try {
    const server = getHorizonServer();
    await server.feeStats();
    return "ok";
  } catch {
    return "error";
  }
}

export async function checkSorobanRpc(): Promise<"ok" | "error"> {
  try {
    const server = getServer();
    await server.getLatestLedger();
    return "ok";
  } catch {
    return "error";
  }
}

export async function runAllChecks(): Promise<HealthCheckResult> {
  const [database, redis, soroban_rpc, horizon] = await Promise.all([
    checkDatabase(),
    checkRedis(),
    checkSorobanRpc(),
    checkHorizon(),
  ]);

  return {
    database,
    redis,
    soroban_rpc,
    horizon,
  };
}

export async function startHealthService() {
  logger.info("Health service started");
  // ...existing code...
}

export function checkHealth() {
  logger.debug("Health check invoked");
  // ...existing code...
}
