'use client';

/**
 * WebSocket client for real-time Stellar event streaming
 * Connects to the PulsarTrack WebSocket server which streams
 * Horizon event data and contract events.
 */

import { z } from 'zod';

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3001';

export type EventType =
  | 'bid_placed'
  | 'auction_created'
  | 'auction_settled'
  | 'campaign_created'
  | 'view_recorded'
  | 'payment_processed'
  | 'consent_updated'
  | 'subscription_created'
  | 'reputation_updated'
  | 'pong'
  | 'disconnected'
  | 'connected'
  | 'authenticated'
  | 'error';

export interface PulsarEvent {
  type: EventType;
  data: Record<string, unknown>;
  timestamp: number;
  txHash?: string;
}

const PulsarEventSchema = z.object({
  type: z.enum([
    'bid_placed',
    'auction_created',
    'auction_settled',
    'campaign_created',
    'view_recorded',
    'payment_processed',
    'consent_updated',
    'subscription_created',
    'reputation_updated',
    'connected',
    'authenticated',
    'error'
  ]),
  data: z.record(z.string(), z.unknown()),
  timestamp: z.number(),
  txHash: z.string().optional(),
});

type EventHandler = (event: PulsarEvent) => void;

export class PulsarWebSocket {
  private ws: WebSocket | null = null;
  private handlers: Map<EventType | 'all', EventHandler[]> = new Map();
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectDelay = 3000;
  private maxReconnectAttempts = 5;
  private reconnectAttempts = 0;
  private url: string;
  private heartbeatInterval: ReturnType<typeof setInterval> | null = null;
  private heartbeatTimeout: ReturnType<typeof setTimeout> | null = null;
  private readonly heartbeatIntervalMs = 30000;
  private readonly heartbeatTimeoutMs = 10000;
  private authToken: string | null = null;
  private authenticated = false;

  constructor(url: string) {
    this.url = url;
  }

  // Token is held in memory only: persisting it to localStorage would make it
  // readable by any injected script (XSS). After a page reload the app must
  // call setAuthToken() again before connecting.
  setAuthToken(token: string): void {
    this.authToken = token;
  }

  connect(): void {
    if (typeof window === 'undefined') return;

    if (this.ws) {
      this.ws.onclose = null;
      this.ws.close();
      this.ws = null;
    }
    this.authenticated = false;

    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        this.reconnectDelay = 3000;
        this.startHeartbeat();

        if (this.authToken) {
          this.ws!.send(JSON.stringify({ type: 'auth', token: this.authToken }));
        }
      };

      this.ws.onmessage = (event) => {
        try {
          const parsed = JSON.parse(event.data);

          if (parsed?.type === 'pong') {
            this.clearHeartbeatTimeout();
            this.emit({ type: 'pong', data: {}, timestamp: Date.now() });
            return;
          }

          if (parsed?.type === 'authenticated') {
            this.authenticated = true;
            this.emit({ type: 'connected', data: parsed.payload || {}, timestamp: Date.now() });
            return;
          }

          const result = PulsarEventSchema.safeParse(parsed);
          if (result.success) {
            this.emit(result.data as PulsarEvent);
          } else {
            console.warn('Invalid WS message:', result.error);
          }
        } catch {
          // ignore malformed JSON messages
        }
      };

      this.ws.onerror = () => {
        this.emit({ type: 'error', data: { msg: 'WebSocket error' }, timestamp: Date.now() });
      };

      this.ws.onclose = () => {
        this.authenticated = false;
        this.stopHeartbeat();
        this.emit({ type: 'disconnected', data: {}, timestamp: Date.now() });
        this.scheduleReconnect();
      };
    } catch {
      this.stopHeartbeat();
      this.scheduleReconnect();
    }
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();

    this.heartbeatInterval = setInterval(() => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        return;
      }

      this.ws.send(JSON.stringify({ type: 'ping' }));
      this.clearHeartbeatTimeout();

      this.heartbeatTimeout = setTimeout(() => {
        this.ws?.close();
      }, this.heartbeatTimeoutMs);
    }, this.heartbeatIntervalMs);
  }

  private clearHeartbeatTimeout(): void {
    if (this.heartbeatTimeout) {
      clearTimeout(this.heartbeatTimeout);
      this.heartbeatTimeout = null;
    }
  }

  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
    this.clearHeartbeatTimeout();
  }

  private scheduleReconnect(): void {
    // Issue #1124: Don't give up after maxReconnectAttempts
    // Continue retrying at maximum backoff interval indefinitely
    // This prevents permanent disconnection after brief outages
    
    // Clear any pending timer before scheduling a new one to prevent accumulation
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.reconnectAttempts++;
      // Exponential backoff up to 30s, then keep retrying at 30s intervals
      this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000);
      this.connect();
    }, this.reconnectDelay);
  }

  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.authenticated = false;
    this.stopHeartbeat();
    if (this.ws) {
      // Detach onclose first: closing would otherwise fire the handler and
      // schedule a fresh reconnect, defeating the explicit disconnect.
      this.ws.onclose = null;
      this.ws.close();
      this.ws = null;
    }
  }

  on(eventType: EventType | 'all', handler: EventHandler): () => void {
    const existing = this.handlers.get(eventType) || [];
    this.handlers.set(eventType, [...existing, handler]);

    // Return unsubscribe function
    return () => {
      const handlers = this.handlers.get(eventType) || [];
      this.handlers.set(
        eventType,
        handlers.filter((h) => h !== handler)
      );
    };
  }

  private emit(event: PulsarEvent): void {
    // Emit to specific handlers
    const specific = this.handlers.get(event.type) || [];
    specific.forEach((h) => h(event));

    // Emit to 'all' handlers
    const all = this.handlers.get('all') || [];
    all.forEach((h) => h(event));
  }

  get isConnected(): boolean {
    return this.authenticated && this.ws?.readyState === WebSocket.OPEN;
  }

  /// Issue #1124: Expose reconnection state for UI
  get connectionState(): { connected: boolean; reconnecting: boolean; attempts: number } {
    return {
      connected: this.isConnected,
      reconnecting: this.reconnectTimer !== null,
      attempts: this.reconnectAttempts,
    };
  }
}

// Singleton instance
let pulsarWs: PulsarWebSocket | null = null;

export function getPulsarWebSocket(): PulsarWebSocket {
  if (!pulsarWs) {
    pulsarWs = new PulsarWebSocket(WS_URL);
  }
  return pulsarWs;
}

export function connectWebSocket(): void {
  getPulsarWebSocket().connect();
}

export function disconnectWebSocket(): void {
  pulsarWs?.disconnect();
}
