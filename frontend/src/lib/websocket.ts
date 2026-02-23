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
  | 'connected'
  | 'error';

export interface PulsarEvent {
  type: EventType;
  data: Record<string, any>;
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
    'error'
  ]),
  data: z.record(z.string(), z.unknown()),
  timestamp: z.number(),
  txHash: z.string().optional(),
});

type EventHandler = (event: PulsarEvent) => void;

class PulsarWebSocket {
  private ws: WebSocket | null = null;
  private handlers: Map<EventType | 'all', EventHandler[]> = new Map();
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectDelay = 3000;
  private maxReconnectAttempts = 5;
  private reconnectAttempts = 0;
  private url: string;

  constructor(url: string) {
    this.url = url;
  }

  connect(): void {
    if (typeof window === 'undefined') return;

    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        this.emit({ type: 'connected', data: {}, timestamp: Date.now() });
      };

      this.ws.onmessage = (event) => {
        try {
          const result = PulsarEventSchema.safeParse(JSON.parse(event.data));
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
        this.scheduleReconnect();
      };
    } catch {
      this.scheduleReconnect();
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) return;

    this.reconnectTimer = setTimeout(() => {
      this.reconnectAttempts++;
      this.connect();
    }, this.reconnectDelay);
  }

  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }
    this.ws?.close();
    this.ws = null;
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
