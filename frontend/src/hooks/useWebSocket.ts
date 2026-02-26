'use client';

import { useEffect, useRef, useState, useCallback } from 'react';
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

  useEffect(() => {
    const ws = getPulsarWebSocket();

    // Subscribe to connected event
    const unsubConnected = ws.on('connected', () => setIsConnected(true));
    const unsubError = ws.on('error', () => setIsConnected(false));

    unsubscribeRefs.current.push(unsubConnected, unsubError);

    // Subscribe to specified events or all
    const eventTypes: Array<EventType | 'all'> = events ? events : ['all'];
    for (const eventType of eventTypes) {
      const unsub = ws.on(eventType, (event) => {
        setLastEvent(event);
        setEventHistory((prev) => [event, ...prev].slice(0, 50));
      });
      unsubscribeRefs.current.push(unsub);
    }

    if (autoConnect) {
      ws.connect();
      setIsConnected(ws.isConnected);
    }

    return () => {
      unsubscribeRefs.current.forEach((unsub) => unsub());
      unsubscribeRefs.current = [];
    };
  }, [autoConnect, events?.join(',')]);

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
