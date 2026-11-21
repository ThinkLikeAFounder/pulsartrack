import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import http from 'http';
import WebSocket from 'ws';
import { setupWebSocketServer, broadcastToChannel, broadcast, MAX_PAYLOAD_SIZE, MAX_MESSAGES_PER_WINDOW } from './websocket-server';
import { createJwt } from '../lib/jwt';

// Mock horizon stream ledgers
vi.mock('./horizon', () => ({
  streamLedgers: vi.fn().mockReturnValue(() => {}),
}));

describe('WebSocket Server Security & Rate Limiting', () => {
  let server: http.Server;
  let wss: ReturnType<typeof setupWebSocketServer>;
  let port: number;

  beforeEach(async () => {
    server = http.createServer();
    wss = setupWebSocketServer(server);
    await new Promise<void>((resolve) => {
      server.listen(0, () => {
        const addr = server.address();
        if (typeof addr === 'object' && addr !== null) {
          port = addr.port;
        }
        resolve();
      });
    });
  });

  afterEach(async () => {
    if (wss) {
      wss.close();
    }
    await new Promise<void>((resolve) => {
      server.close(() => resolve());
    });
  });

  it('rejects oversized payload gracefully', async () => {
    const ws = new WebSocket(`ws://localhost:${port}/ws`);

    await new Promise<void>((resolve) => {
      ws.on('open', resolve);
    });

    // Create payload larger than MAX_PAYLOAD_SIZE
    const oversizedBuffer = Buffer.alloc(MAX_PAYLOAD_SIZE + 1024, 'a');

    const closeOrError = new Promise<void>((resolve) => {
      ws.on('close', () => resolve());
      ws.on('error', () => resolve());
    });

    ws.send(oversizedBuffer);

    await closeOrError;
    expect(ws.readyState).toBeGreaterThanOrEqual(WebSocket.CLOSING);
  });

  it('enforces per-connection message rate limit', async () => {
    const ws = new WebSocket(`ws://localhost:${port}/ws`);

    await new Promise<void>((resolve) => {
      ws.on('open', resolve);
    });

    const token = createJwt({ sub: 'GABC1234567890123456789012345678901234567890123456789012' });

    // Authenticate client
    ws.send(JSON.stringify({ type: 'auth', token }));

    let rateLimitExceededReceived = false;

    ws.on('message', (data) => {
      try {
        const msg = JSON.parse(data.toString());
        if (msg.type === 'error' && msg.payload?.message === 'Rate limit exceeded') {
          rateLimitExceededReceived = true;
        }
      } catch {}
    });

    // Send messages exceeding MAX_MESSAGES_PER_WINDOW
    for (let i = 0; i <= MAX_MESSAGES_PER_WINDOW + 5; i++) {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'ping' }));
      }
    }

    await new Promise((resolve) => setTimeout(resolve, 200));

    expect(rateLimitExceededReceived).toBe(true);
    ws.close();
  });

  describe('broadcastToChannel', () => {
    it('does not throw when called with no clients', () => {
      expect(() => {
        broadcastToChannel('ledger', {
          type: 'LEDGER_CLOSED',
          payload: {},
          timestamp: Date.now(),
        });
      }).not.toThrow();
    });
  });

  describe('broadcast', () => {
    it('does not throw when called with no clients', () => {
      expect(() => {
        broadcast({
          type: 'platform_event',
          payload: {},
          timestamp: Date.now(),
        });
      }).not.toThrow();
    });
  });
});
