import Redis from "ioredis";
import { logger } from "../lib/logger";
import { loadEnv } from "./env";

const redisClient = new Redis(loadEnv().REDIS_URL || "redis://localhost:6379", {
  enableOfflineQueue: false,
  maxRetriesPerRequest: 1,
});

redisClient.on("connect", () => logger.info("[Redis] Connected"));
redisClient.on("error", (err) => logger.error(err, "[Redis] Error"));

export default redisClient;
