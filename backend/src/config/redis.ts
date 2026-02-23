import Redis from "ioredis";
import { logger } from "../lib/logger";

const redisClient = new Redis(
  process.env.REDIS_URL || "redis://localhost:6379",
  {
    enableOfflineQueue: false,
    maxRetriesPerRequest: 1,
  },
);

redisClient.on("connect", () => logger.info("[Redis] Connected"));
redisClient.on("error", (err: any) => logger.error(err, "[Redis] Error"));

export default redisClient;
