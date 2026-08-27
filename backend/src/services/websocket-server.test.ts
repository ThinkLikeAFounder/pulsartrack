import { describe, it, expect, vi, beforeEach } from 'vitest';
import { broadcastToChannel, broadcast } from './websocket-server';

// Mock ws
vi.mock('ws', () => {
  const MockWebSocket = vi.fn().mockImplementation(() => ({
    readyState: 1, // OPEN
    send: vi.fn(),
    close: vi.fn(),
    on: vi.fn(),
  }));
  // `ws` exposes OPEN as a static on the constructor; the production code
  // compares against it, so the mock has to carry it too.
  return {
    WebSocketServer: vi.fn(),
    WebSocket: Object.assign(MockWebSocket, { OPEN: 1 }),
  };
});

vi.mock('./horizon', () => ({
  streamLedgers: vi.fn().mockReturnValue(vi.fn()),
}));

vi.mock('../lib/logger', () => ({
  logger: { info: vi.fn(), error: vi.fn(), warn: vi.fn(), debug: vi.fn() },
}));

vi.mock('../lib/jwt', () => ({
  decodeJwt: vi.fn(),
  createJwt: vi.fn(),
  TOKEN_EXPIRY: 3600,
}));

describe('websocket-server', () => {
  // Note: setupWebSocketServer requires an HTTP server and full WS handshake.
  // These tests cover the exported broadcast helpers which are the
  // security-relevant public API surface.
  // Full integration tests for JWT auth over WS are best done in an
  // e2e suite with a real HTTP server.

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
