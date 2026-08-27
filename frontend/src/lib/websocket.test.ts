import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';

describe('PulsarWebSocket', () => {
  let PulsarWebSocket: typeof import('./websocket').PulsarWebSocket;
  let getPulsarWebSocket: typeof import('./websocket').getPulsarWebSocket;
  let connectWebSocket: typeof import('./websocket').connectWebSocket;
  let disconnectWebSocket: typeof import('./websocket').disconnectWebSocket;

  interface ControllableMockWs {
    onopen: (() => void) | null;
    onmessage: ((event: { data: string }) => void) | null;
    onerror: (() => void) | null;
    onclose: (() => void) | null;
    readyState: number;
    url: string;
    close: () => void;
    send: (data: string) => void;
    sentMessages: string[];
  }

  let currentWs: ControllableMockWs | null = null;
  let pendingWsCreation: ((ws: ControllableMockWs) => void) | null = null;

  class MockWs implements ControllableMockWs {
    static readonly CONNECTING = 0;
    static readonly OPEN = 1;
    static readonly CLOSING = 2;
    static readonly CLOSED = 3;

    url: string;
    readyState = 0;
    onopen: (() => void) | null = null;
    onmessage: ((event: { data: string }) => void) | null = null;
    onerror: (() => void) | null = null;
    onclose: (() => void) | null = null;
    sentMessages: string[] = [];

    constructor(url: string) {
      this.url = url;
      MockWs.register(this);
    }

    private static register(ws: MockWs): void {
      currentWs = ws;
      if (pendingWsCreation) {
        pendingWsCreation(ws);
      }
    }

    send(data: string): void {
      this.sentMessages.push(data);
    }

    close(): void {
      this.readyState = MockWs.CLOSED;
      if (this.onclose) this.onclose();
    }
  }

  // Records every construction while still returning a real MockWs instance
  // (prototype methods intact), so the source's `new WebSocket(url)` and its
  // static `WebSocket.OPEN` reads both behave like the real API.
  const constructionSpy = vi.fn();

  class SpyWs extends MockWs {
    constructor(url: string) {
      super(url);
      constructionSpy(url);
    }
  }

  const mockWebSocketClass = Object.assign(SpyWs, { mock: constructionSpy.mock });

  function triggerOpen() {
    if (!currentWs) throw new Error('No WebSocket created yet');
    currentWs.readyState = 1;
    if (currentWs.onopen) currentWs.onopen();
  }

  function triggerMessage(data: unknown) {
    if (!currentWs) throw new Error('No WebSocket created yet');
    if (currentWs.onmessage) {
      currentWs.onmessage({ data: JSON.stringify(data) });
    }
  }

  function triggerRawMessage(rawData: string) {
    if (!currentWs) throw new Error('No WebSocket created yet');
    if (currentWs.onmessage) {
      currentWs.onmessage({ data: rawData });
    }
  }

  function triggerError() {
    if (!currentWs) throw new Error('No WebSocket created yet');
    if (currentWs.onerror) currentWs.onerror();
  }

  function triggerClose() {
    if (!currentWs) throw new Error('No WebSocket created yet');
    currentWs.readyState = 3;
    if (currentWs.onclose) currentWs.onclose();
  }

  beforeEach(async () => {
    vi.useFakeTimers();
    currentWs = null;
    pendingWsCreation = null;
    constructionSpy.mockClear();

    globalThis.WebSocket = mockWebSocketClass as unknown as typeof WebSocket;

    // Reset singleton between tests
    vi.resetModules();
    const mod = await import('./websocket');
    PulsarWebSocket = mod.PulsarWebSocket;
    getPulsarWebSocket = mod.getPulsarWebSocket;
    connectWebSocket = mod.connectWebSocket;
    disconnectWebSocket = mod.disconnectWebSocket;
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  describe('connect', () => {
    it('creates a WebSocket with the correct URL', () => {
      const client = new PulsarWebSocket('ws://test.example.com:4000');
      client.connect();
      expect(constructionSpy).toHaveBeenCalledTimes(1);
      expect(constructionSpy).toHaveBeenCalledWith('ws://test.example.com:4000');
    });

    it('emits "connected" event and resets reconnect counters when WebSocket opens', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      const connectedHandler = vi.fn();
      client.on('connected', connectedHandler);

      client.connect();
      expect(connectedHandler).not.toHaveBeenCalled();

      triggerOpen();

      expect(connectedHandler).toHaveBeenCalledTimes(1);
      const event = connectedHandler.mock.calls[0][0];
      expect(event.type).toBe('connected');
      expect(event.data).toEqual({});
      expect(typeof event.timestamp).toBe('number');
      expect(client.isConnected).toBe(true);
    });

    it('closes existing connection before opening a new one (no reconnect loop)', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      client.connect();
      triggerOpen();

      const firstWs = currentWs;
      expect(firstWs?.readyState).toBe(1);

      client.connect();

      // Previous onclose should be nullified so it doesn't schedule another reconnect
      expect(firstWs?.onclose).toBeNull();
      expect(constructionSpy).toHaveBeenCalledTimes(2);
    });

    it('isConnected reflects readyState correctly', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      expect(client.isConnected).toBe(false);

      client.connect();
      expect(client.isConnected).toBe(false);

      triggerOpen();
      expect(client.isConnected).toBe(true);

      triggerClose();
      expect(client.isConnected).toBe(false);
    });
  });

  describe('message dispatch', () => {
    it('dispatches a valid event to specific and "all" handlers', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      client.connect();
      triggerOpen();

      const bidHandler = vi.fn();
      const allHandler = vi.fn();
      client.on('bid_placed', bidHandler);
      client.on('all', allHandler);

      const payload = {
        type: 'bid_placed',
        data: { amount: '100', bidder: 'GABC' },
        timestamp: 1700000000000,
        txHash: 'abc123',
      };
      triggerMessage(payload);

      expect(bidHandler).toHaveBeenCalledTimes(1);
      expect(bidHandler.mock.calls[0][0]).toEqual(payload);
      // Handlers were registered after 'connected' fired, so 'all' only sees
      // the bid_placed event.
      expect(allHandler).toHaveBeenCalledTimes(1);
      expect(allHandler.mock.calls[0][0]).toEqual(payload);
    });

    it('handles pong messages and clears heartbeat timeout', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      const pongHandler = vi.fn();
      client.on('pong', pongHandler);

      client.connect();
      triggerOpen();

      const pingIntervalMs = 30000;
      vi.advanceTimersByTime(pingIntervalMs);

      // After heartbeat interval, ping should have been sent
      expect(currentWs?.sentMessages).toContain(JSON.stringify({ type: 'ping' }));

      // Trigger pong back from server
      triggerMessage({ type: 'pong' });
      expect(pongHandler).toHaveBeenCalledTimes(1);

      // Advance past the heartbeat timeout (10s after ping was sent) - since we got pong,
      // the connection should NOT have closed
      vi.advanceTimersByTime(10000);
      expect(currentWs?.readyState).toBe(1);
    });

    it('ignores malformed JSON messages silently', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      const errorHandler = vi.fn();
      const allHandler = vi.fn();
      client.on('error', errorHandler);
      client.on('all', allHandler);

      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      client.connect();
      triggerOpen();

      triggerRawMessage('this is not json {{');

      // No error event, no dispatch beyond connected
      expect(errorHandler).not.toHaveBeenCalled();
      expect(allHandler).toHaveBeenCalledTimes(1); // only connected
      expect(warnSpy).not.toHaveBeenCalled();
      warnSpy.mockRestore();
    });

    it('warns on schema-invalid messages (wrong type or missing fields)', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      const bidHandler = vi.fn();
      client.on('bid_placed', bidHandler);

      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      client.connect();
      triggerOpen();

      triggerMessage({
        type: 'invalid_event_type_xyz',
        data: { foo: 'bar' },
        timestamp: 1700000000000,
      });

      expect(bidHandler).not.toHaveBeenCalled();
      expect(warnSpy).toHaveBeenCalledWith(
        'Invalid WS message:',
        expect.any(Object),
      );
      warnSpy.mockRestore();
    });

    it('emits error event on websocket onerror', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      const errorHandler = vi.fn();
      client.on('error', errorHandler);

      client.connect();
      triggerError();

      expect(errorHandler).toHaveBeenCalledTimes(1);
      const event = errorHandler.mock.calls[0][0];
      expect(event.type).toBe('error');
      expect(event.data.msg).toBe('WebSocket error');
    });
  });

  describe('reconnect on drop', () => {
    it('schedules a reconnect after connection closes', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      client.connect();
      triggerOpen();

      expect(constructionSpy).toHaveBeenCalledTimes(1);

      triggerClose();
      expect(client.isConnected).toBe(false);

      // Reconnect is scheduled at 3s delay
      vi.advanceTimersByTime(2999);
      expect(constructionSpy).toHaveBeenCalledTimes(1);

      vi.advanceTimersByTime(1);
      expect(constructionSpy).toHaveBeenCalledTimes(2);
    });

    it('uses exponential backoff for subsequent reconnect attempts', () => {
      const client = new PulsarWebSocket('ws://test.example.com');

      client.connect();
      triggerOpen();
      expect(constructionSpy).toHaveBeenCalledTimes(1);

      // First close -> reconnect at 3s
      triggerClose();
      vi.advanceTimersByTime(3000);
      expect(constructionSpy).toHaveBeenCalledTimes(2);
      triggerOpen();

      // Second close -> reconnect at 6s (3 * 2)
      triggerClose();
      vi.advanceTimersByTime(6000);
      expect(constructionSpy).toHaveBeenCalledTimes(3);
      triggerOpen();

      // Third close -> reconnect at 12s (6 * 2)
      triggerClose();
      vi.advanceTimersByTime(12000);
      expect(constructionSpy).toHaveBeenCalledTimes(4);
    });

    it('stops reconnecting after maxReconnectAttempts (5)', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      client.connect();
      triggerOpen();

      // Never re-open: a successful open resets reconnectAttempts, so the cap
      // is only reachable across consecutive failed reconnects.
      for (let i = 0; i < 5; i++) {
        triggerClose();
        vi.advanceTimersByTime(3000 * Math.pow(2, i));
      }

      // 1 initial + 5 reconnect attempts
      expect(constructionSpy).toHaveBeenCalledTimes(6);

      // 6th close is past maxReconnectAttempts and must NOT reconnect
      triggerClose();
      vi.advanceTimersByTime(60000);

      expect(constructionSpy).toHaveBeenCalledTimes(6);
    });

    it('resets reconnect counters after a successful reconnection', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      client.connect();
      triggerOpen();

      // Two failures with backoff
      triggerClose();
      vi.advanceTimersByTime(3000);
      triggerOpen();

      triggerClose();
      vi.advanceTimersByTime(6000);
      triggerOpen(); // should reset counters

      // Next close should start at 3s again, not 12s
      triggerClose();
      vi.advanceTimersByTime(3000);
      expect(constructionSpy).toHaveBeenCalledTimes(4);
    });

    it('disconnect cancels pending reconnect timers and closes ws', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      client.connect();
      triggerOpen();

      triggerClose();
      // Reconnect is now pending at 3s

      client.disconnect();

      vi.advanceTimersByTime(30000);
      // No new WebSocket should have been created (the pending reconnect was cleared)
      expect(constructionSpy).toHaveBeenCalledTimes(1);
    });
  });

  describe('on() unsubscribe', () => {
    it('returned function removes the specific handler without affecting others', () => {
      const client = new PulsarWebSocket('ws://test.example.com');
      client.connect();
      triggerOpen();

      const h1 = vi.fn();
      const h2 = vi.fn();

      const unsub1 = client.on('bid_placed', h1);
      client.on('bid_placed', h2);

      triggerMessage({
        type: 'bid_placed',
        data: { amount: '50' },
        timestamp: 1700000000000,
      });
      expect(h1).toHaveBeenCalledTimes(1);
      expect(h2).toHaveBeenCalledTimes(1);

      unsub1();

      triggerMessage({
        type: 'bid_placed',
        data: { amount: '75' },
        timestamp: 1700000000001,
      });
      expect(h1).toHaveBeenCalledTimes(1);
      expect(h2).toHaveBeenCalledTimes(2);
    });
  });

  describe('singleton getPulsarWebSocket / connectWebSocket / disconnectWebSocket', () => {
    it('getPulsarWebSocket returns the same instance across calls', () => {
      const a = getPulsarWebSocket();
      const b = getPulsarWebSocket();
      expect(a).toBe(b);
    });

    it('connectWebSocket and disconnectWebSocket operate on the singleton', () => {
      connectWebSocket();
      expect(constructionSpy).toHaveBeenCalledTimes(1);

      triggerOpen();
      const ws = getPulsarWebSocket();
      expect(ws.isConnected).toBe(true);

      disconnectWebSocket();
      expect(currentWs?.readyState).toBe(3);
    });
  });
});
