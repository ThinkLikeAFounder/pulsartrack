import 'dotenv/config';
import { createServer } from 'http';
import app, { redisClient } from './app';
import { setupWebSocketServer } from './services/websocket-server';
import { checkDbConnection } from './config/database';
import prisma from './db/prisma';

const PORT = parseInt(process.env.PORT || '4000', 10);

// Create HTTP server for both REST and WebSocket
const server = createServer(app);

// Attach WebSocket server
setupWebSocketServer(server);

// Start server
async function start() {
  // Verify database connection — fail hard in production
  const dbOk = await checkDbConnection();
  if (!dbOk) {
    if (process.env.NODE_ENV === 'production') {
      console.error('[DB] PostgreSQL connection failed — aborting in production');
      process.exit(1);
    }
    console.warn('[DB] Could not connect to PostgreSQL — running without DB');
  } else {
    console.log('[DB] PostgreSQL connected');
  }

  // Verify Prisma client connectivity
  try {
    await prisma.$connect();
    console.log('[DB] Prisma client connected');
  } catch (err) {
    if (process.env.NODE_ENV === 'production') {
      console.error('[DB] Prisma connection failed — aborting in production');
      process.exit(1);
    }
    console.warn('[DB] Prisma client unavailable — running without ORM');
  }

  server.listen(PORT, () => {
    console.log(`[PulsarTrack API] Listening on http://localhost:${PORT}`);
    console.log(`[PulsarTrack WS]  WebSocket on ws://localhost:${PORT}/ws`);
    console.log(`[Network]         ${process.env.STELLAR_NETWORK || 'testnet'}`);
  });
}

if (process.env.NODE_ENV !== 'test') {
  start().catch((err) => {
    console.error('Failed to start server:', err);
    process.exit(1);
  });
}

export { server };
