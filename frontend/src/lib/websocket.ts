'use client';

/**
 * WebSocket client for real-time Stellar event streaming
 * Connects to the PulsarTrack WebSocket server which streams
 * Horizon event data and contract events.
 */

import { z } from 'zod';

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3001';
const AUTH_TOKEN_KEY = 'pulsar_auth_token';

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

  constructor(url: string) {
    this.url = url;
  }

  setAuthToken(token: string): void {
    this.authToken = token;
    if (typeof window !== 'undefined') {
      localStorage.setItem(AUTH_TOKEN_KEY, token);
    }
  }

  private loadAuthToken(): string | null {
    if (this.authToken) return this.authToken;
    if (typeof window !== 'undefined') {
      this.authToken = localStorage.getItem(AUTH_TOKEN_KEY);
      return this.authToken;
    }
    return null;
  }

  connect(): void {
    if (typeof window === 'undefined') return;

    if (this.ws) {
      this.ws.onclose = null;
      this.ws.close();
      this.ws = null;
    }

    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        this.reconnectDelay = 3000;
        this.startHeartbeat();

        const token = this.loadAuthToken();
        if (token) {
          this.ws!.send(JSON.stringify({ type: 'auth', token }));
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
    if (this.reconnectAttempts >= this.maxReconnectAttempts) return;

    // Clear any pending timer before scheduling a new one to prevent accumulation
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.reconnectAttempts++;
      this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000); // exponential backoff
      this.connect();
    }, this.reconnectDelay);
  }

  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
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
    return this.ws?.readyState === WebSocket.OPEN;
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
