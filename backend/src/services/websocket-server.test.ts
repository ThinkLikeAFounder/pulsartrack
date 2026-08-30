import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import http from 'http';
import WebSocket from 'ws';
import { setupWebSocketServer, MAX_PAYLOAD_SIZE, MAX_MESSAGES_PER_WINDOW } from './websocket-server';

// Mock horizon stream ledgers
vi.mock('./horizon', () => ({
  streamLedgers: vi.fn().mockReturnValue(() => {}),
}));

describe('WebSocket Server Security & Rate Limiting', () => {
  let server: http.Server;
  let wss: any;
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

    const closePromise = new Promise<number>((resolve) => {
      ws.on('close', (code) => resolve(code));
    });

    // Send messages up to and exceeding MAX_MESSAGES_PER_WINDOW
    for (let i = 0; i <= MAX_MESSAGES_PER_WINDOW; i++) {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'ping' }));
      }
    }

    const code = await closePromise;
    expect(code).toBe(1008);
  });
});
