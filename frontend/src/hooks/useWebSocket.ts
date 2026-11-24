'use client';

import { useEffect, useMemo, useRef, useState, useCallback } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { getPulsarWebSocket, Channel, EventType, PulsarEvent } from '../lib/websocket';

interface UseWebSocketOptions {
  /**
   * Call ws.connect() on mount. Leave false when the connection lifecycle is
   * owned elsewhere (see WebSocketLifecycle in components/providers.tsx) —
   * connect() tears down and reopens the shared socket.
   */
  autoConnect?: boolean;
  events?: EventType[];
  /** Server broadcast channels to subscribe to while mounted. */
  channels?: Channel[];
}

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const { autoConnect = true, events, channels } = options;
  // The shared socket may already be authenticated when this hook mounts
  // (its 'connected' event has then already fired), so seed from it.
  const [isConnected, setIsConnected] = useState(() => getPulsarWebSocket().isConnected);
  const [lastEvent, setLastEvent] = useState<PulsarEvent | null>(null);
  const [eventHistory, setEventHistory] = useState<PulsarEvent[]>([]);
  const unsubscribeRefs = useRef<Array<() => void>>([]);

  // Stable, content-based key so the effect only re-runs when the requested
  // event types actually change (not when the array reference changes).
  const eventsKey = useMemo(() => (events ? events.join(',') : ''), [events]);
  const channelsKey = useMemo(() => (channels ? channels.join(',') : ''), [channels]);

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

    if (channelsKey) {
      for (const channel of channelsKey.split(',') as Channel[]) {
        unsubscribeRefs.current.push(ws.subscribe(channel));
      }
    }

    // Connection state is driven by the 'connected'/'disconnected'/'error'
    // event handlers subscribed above. ws.connect() emits 'connected' only
    // once the server confirms authentication, so we rely on those callbacks
    // rather than reading ws.isConnected synchronously here.
    if (autoConnect) {
      ws.connect();
    }

    return () => {
      unsubscribeRefs.current.forEach((unsub) => unsub());
      unsubscribeRefs.current = [];
    };
  }, [autoConnect, eventsKey, channelsKey]);

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

const AUCTION_EVENTS: EventType[] = ['bid_placed', 'auction_created', 'auction_settled'];
const CAMPAIGN_EVENTS: EventType[] = ['campaign_created', 'view_recorded', 'payment_processed'];
const AUCTION_CHANNELS: Channel[] = ['auctions'];
const CAMPAIGN_CHANNELS: Channel[] = ['campaigns'];

/**
 * Hook for real-time auction events. The shared connection is opened by
 * WebSocketLifecycle, so this only subscribes.
 */
export function useAuctionEvents() {
  return useWebSocket({ autoConnect: false, events: AUCTION_EVENTS, channels: AUCTION_CHANNELS });
}

/**
 * Hook for real-time campaign events. The shared connection is opened by
 * WebSocketLifecycle, so this only subscribes.
 */
export function useCampaignEvents() {
  return useWebSocket({ autoConnect: false, events: CAMPAIGN_EVENTS, channels: CAMPAIGN_CHANNELS });
}

/**
 * Refetch auction queries whenever a live auction event arrives, so bids and
 * settlements from other users show up without a manual refresh.
 */
export function useLiveAuctionUpdates() {
  const queryClient = useQueryClient();
  const auctionEvents = useAuctionEvents();
  const { lastEvent } = auctionEvents;

  useEffect(() => {
    if (!lastEvent) return;
    queryClient.invalidateQueries({ queryKey: ['open_auctions'] });
    queryClient.invalidateQueries({ queryKey: ['publisher_auctions'] });
    if (lastEvent.type === 'auction_created') {
      // New auction changes get_auction_count, which the lists key on.
      queryClient.invalidateQueries({
        predicate: (q) => q.queryKey[0] === 'contract' && q.queryKey[2] === 'get_auction_count',
      });
    }
  }, [lastEvent, queryClient]);

  return auctionEvents;
}

/**
 * Refetch campaign queries whenever a live campaign event arrives (new
 * campaigns, recorded views, payments) so status and stats stay current.
 */
export function useLiveCampaignUpdates() {
  const queryClient = useQueryClient();
  const campaignEvents = useCampaignEvents();
  const { lastEvent } = campaignEvents;

  useEffect(() => {
    if (!lastEvent) return;
    queryClient.invalidateQueries({ queryKey: ['advertiser_campaigns'] });
    queryClient.invalidateQueries({
      predicate: (q) =>
        q.queryKey[0] === 'contract' &&
        (q.queryKey[2] === 'get_campaign_count' || q.queryKey[2] === 'get_advertiser_stats'),
    });
  }, [lastEvent, queryClient]);

  return campaignEvents;
}
