'use client';

import { useEffect, useMemo, useRef, useState, useCallback } from 'react';
import { getPulsarWebSocket, EventType, PulsarEvent } from '../lib/websocket';

interface UseWebSocketOptions {
  autoConnect?: boolean;
  events?: EventType[];
}

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const { autoConnect = true, events } = options;
  const [isConnected, setIsConnected] = useState(false);
  const [lastEvent, setLastEvent] = useState<PulsarEvent | null>(null);
  const [eventHistory, setEventHistory] = useState<PulsarEvent[]>([]);
  const unsubscribeRefs = useRef<Array<() => void>>([]);

  // Stable, content-based key so the effect only re-runs when the requested
  // event types actually change (not when the array reference changes).
  const eventsKey = useMemo(() => (events ? events.join(',') : ''), [events]);

  useEffect(() => {
    const ws = getPulsarWebSocket();

    // Subscribe to connected event
    const unsubConnected = ws.on('connected', () => setIsConnected(true));
    const unsubError = ws.on('error', () => setIsConnected(false));
    const unsubDisconnected = ws.on('disconnected', () => setIsConnected(false));

    unsubscribeRefs.current.push(unsubConnected, unsubError, unsubDisconnected);

    // Subscribe to specified events or all
    const eventTypes: Array<EventType | 'all'> = eventsKey
      ? (eventsKey.split(',') as EventType[])
      : ['all'];
    for (const eventType of eventTypes) {
      const unsub = ws.on(eventType, (event) => {
        setLastEvent(event);
        setEventHistory((prev) => [event, ...prev].slice(0, 50));
      });
      unsubscribeRefs.current.push(unsub);
    }

    // Connection state is driven by the 'connected'/'disconnected'/'error'
    // event handlers subscribed above. ws.connect() emits 'connected' on open,
    // so we rely on those callbacks rather than reading ws.isConnected
    // synchronously here.
    if (autoConnect) {
      ws.connect();
    }

    return () => {
      unsubscribeRefs.current.forEach((unsub) => unsub());
      unsubscribeRefs.current = [];
    };
  }, [autoConnect, eventsKey]);

  const clearHistory = useCallback(() => {
    setEventHistory([]);
    setLastEvent(null);
  }, []);

  return {
    isConnected,
    lastEvent,
    eventHistory,
    clearHistory,
  };
}

/**
 * Hook for real-time auction events
 */
export function useAuctionEvents() {
  return useWebSocket({ events: ['bid_placed', 'auction_created', 'auction_settled'] });
}

/**
 * Hook for real-time campaign events
 */
export function useCampaignEvents() {
  return useWebSocket({ events: ['campaign_created', 'view_recorded', 'payment_processed'] });
}
