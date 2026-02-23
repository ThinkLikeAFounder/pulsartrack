import "dotenv/config";
import express from "express";
import cors from "cors";
import helmet from "helmet";
import morgan from "morgan";
import { createServer } from "http";
import apiRoutes from "./api/routes";
import {
  errorHandler,
  rateLimit,
  configureRateLimiters,
} from "./middleware/auth";
import { setupWebSocketServer } from "./services/websocket-server";
import { checkDbConnection } from "./config/database";
import prisma from "./db/prisma";
import redisClient from "./config/redis";

const app = express();
const PORT = parseInt(process.env.PORT || "4000", 10);

// Initialize Redis-backed rate limiters
configureRateLimiters(redisClient);

// Middleware
app.use(helmet());
app.use(
  cors({
    origin: process.env.CORS_ORIGIN || "http://localhost:3000",
    credentials: true,
  }),
);
app.use(morgan("combined"));
app.use(express.json({ limit: "10mb" }));
app.use(rateLimit());

// API routes
app.use("/api", apiRoutes);

// 404 handler
app.use((_req, res) => {
  res.status(404).json({ error: "Route not found" });
});

// Error handler
app.use(errorHandler);

// Create HTTP server for both REST and WebSocket
const server = createServer(app);

// Attach WebSocket server
setupWebSocketServer(server);

// Start server
async function start() {
  // Verify database connection — fail hard in production
  const dbOk = await checkDbConnection();
  if (!dbOk) {
    if (process.env.NODE_ENV === "production") {
      console.error(
        "[DB] PostgreSQL connection failed — aborting in production",
      );
      process.exit(1);
    }
    console.warn("[DB] Could not connect to PostgreSQL — running without DB");
  } else {
    console.log("[DB] PostgreSQL connected");
  }

  // Verify Prisma client connectivity
  try {
    await prisma.$connect();
    console.log("[DB] Prisma client connected");
  } catch (err) {
    if (process.env.NODE_ENV === "production") {
      console.error("[DB] Prisma connection failed — aborting in production");
      process.exit(1);
    }
    console.warn("[DB] Prisma client unavailable — running without ORM");
  }

  server.listen(PORT, () => {
    console.log(`[PulsarTrack API] Listening on http://localhost:${PORT}`);
    console.log(`[PulsarTrack WS]  WebSocket on ws://localhost:${PORT}/ws`);
    console.log(
      `[Network]         ${process.env.STELLAR_NETWORK || "testnet"}`,
    );
  });
}

if (process.env.NODE_ENV !== 'test') {
  start().catch((err) => {
    console.error('Failed to start server:', err);
    process.exit(1);
  });
}

export { server };
