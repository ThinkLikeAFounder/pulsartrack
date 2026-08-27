import "dotenv/config";
import { createServer } from "http";
import app from "./app";
import { setupWebSocketServer } from "./services/websocket-server";
import pool, { checkDbConnection } from "./config/database";
import { validateContractIds } from "./config/stellar";
import prisma from "./db/prisma";
import redisClient from "./config/redis";
import { validateSimulationAccount } from "./services/soroban-client";
import { EnvValidationError, loadEnv } from "./config/env";
import { logger } from "./lib/logger";

const PORT = parseInt(process.env.PORT || "4000", 10);

const server = createServer(app);

setupWebSocketServer(server);

async function closeResources() {
  try {
    await prisma.$disconnect();
    logger.info("[PulsarTrack] Prisma disconnected");
  } catch (err) {
    logger.error({ err }, "[PulsarTrack] Prisma disconnect error");
  }

  try {
    await pool.end();
    logger.info("[PulsarTrack] PostgreSQL pool closed");
  } catch (err) {
    logger.error({ err }, "[PulsarTrack] PostgreSQL disconnect error");
  }

  try {
    if (redisClient.status !== "end") {
      await redisClient.quit();
      logger.info("[PulsarTrack] Redis disconnected");
    }
  } catch (err) {
    logger.error({ err }, "[PulsarTrack] Redis disconnect error");
  }
}

async function shutdown(exitCode: number, closeServer = false) {
  if (closeServer && server.listening) {
    await new Promise<void>((resolve, reject) => {
      server.close((err) => {
        if (err) {
          reject(err);
          return;
        }
        logger.info("[PulsarTrack] HTTP server closed");
        resolve();
      });
    });
  }

  await closeResources();
  return exitCode;
}

async function gracefulShutdown(signal: string) {
  logger.info(`[PulsarTrack] Received ${signal}, shutting down gracefully...`);

  const forceShutdownTimer = setTimeout(() => {
    logger.error("[PulsarTrack] Forced shutdown after timeout");
    process.exit(1);
  }, 10000);

  try {
    const exitCode = await shutdown(0, true);
    clearTimeout(forceShutdownTimer);
    process.exit(exitCode);
  } catch (err) {
    clearTimeout(forceShutdownTimer);
    logger.error({ err }, "[PulsarTrack] Graceful shutdown failed");
    process.exit(1);
  }
}

process.on("SIGTERM", () => gracefulShutdown("SIGTERM"));
process.on("SIGINT", () => gracefulShutdown("SIGINT"));

async function start() {
  // Validate the whole environment up front so a misconfigured deploy fails
  // here, naming the offending variables, rather than at request time.
  loadEnv();

  validateContractIds();

  await validateSimulationAccount();

  const dbOk = await checkDbConnection();
  if (!dbOk) {
    if (process.env.NODE_ENV === "production") {
      logger.fatal(
        "[DB] PostgreSQL connection failed — aborting in production",
      );
      process.exit(1);
    }
    logger.warn("[DB] Could not connect to PostgreSQL — running without DB");
  } else {
    logger.info("[DB] PostgreSQL connected");
  }

  try {
    await prisma.$connect();
    logger.info("[DB] Prisma client connected");
  } catch (err) {
    if (process.env.NODE_ENV === "production") {
      logger.fatal("[DB] Prisma connection failed — aborting in production");
      process.exit(1);
    }
    logger.warn("[DB] Prisma client unavailable — running without ORM");
  }

  server.listen(PORT, () => {
    logger.info(`[PulsarTrack API] Listening on http://localhost:${PORT}`);
    logger.info(`[PulsarTrack WS]  WebSocket on ws://localhost:${PORT}/ws`);
    logger.info(
      `[Network]         ${process.env.STELLAR_NETWORK || "testnet"}`,
    );
  });
}

if (process.env.NODE_ENV !== "test") {
  start().catch(async (err) => {
    if (err instanceof EnvValidationError) {
      // Config is broken before anything was opened — report and exit
      // without attempting a resource teardown.
      console.error(err.message);
      process.exit(1);
    }
    console.error("Failed to start server:", err);
    const exitCode = await shutdown(1);
    process.exit(exitCode);
  });
}

export { server };
