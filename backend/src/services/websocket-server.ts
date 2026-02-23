import { WebSocketServer, WebSocket } from "ws";
import { IncomingMessage, Server } from "http";
import { streamLedgers } from "./horizon";
import { logger } from "../lib/logger";

interface PulsarEvent {
  type: string;
  payload: any;
  timestamp: number;
  txHash?: string;
}

const clients = new Set<WebSocket>();

const INITIAL_BACKOFF_MS = 1000;
const MAX_BACKOFF_MS = 30000;
let currentBackoff = INITIAL_BACKOFF_MS;
let stopStream: (() => void) | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

function startLedgerStream(): void {
  stopStream = streamLedgers(
    (ledger) => {
      // Stream is alive — reset backoff on successful message
      currentBackoff = INITIAL_BACKOFF_MS;

      broadcast({
        type: "LEDGER_CLOSED",
        payload: {
          sequence: ledger.sequence,
          closed_at: ledger.closed_at,
          transactionCount: ledger.transaction_count,
        },
        timestamp: Date.now(),
      });
    },
    (err: any) => {
      logger.error(err, "[WS] Ledger stream error");
      scheduleReconnect();
    },
  );
}

function scheduleReconnect(): void {
  if (reconnectTimer) return;

  broadcast({
    type: "reconnecting",
    payload: {
      message: "Horizon stream dropped, reconnecting...",
      retryMs: currentBackoff,
    },
    timestamp: Date.now(),
  });

  logger.info(`[WS] Reconnecting in ${currentBackoff}ms...`);

  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;

    // Clean up previous stream
    if (stopStream) {
      try {
        stopStream();
      } catch {
        /* already closed */
      }
      stopStream = null;
    }

    startLedgerStream();

    broadcast({
      type: "reconnected",
      payload: { message: "Horizon stream resumed" },
      timestamp: Date.now(),
    });

    // Exponential backoff for next failure
    currentBackoff = Math.min(currentBackoff * 2, MAX_BACKOFF_MS);
  }, currentBackoff);
}

export function setupWebSocketServer(server: Server): WebSocketServer {
  const wss = new WebSocketServer({ server, path: "/ws" });

  wss.on("connection", (ws: WebSocket, req: IncomingMessage) => {
    clients.add(ws);
    logger.info(`[WS] Client connected. Total: ${clients.size}`);

    // Send welcome message
    sendToClient(ws, {
      type: "connected",
      payload: { message: "Connected to PulsarTrack WebSocket server" },
      timestamp: Date.now(),
    });

    ws.on("close", () => {
      clients.delete(ws);
      logger.info(`[WS] Client disconnected. Total: ${clients.size}`);
    });

    ws.on("error", (err) => {
      logger.error(err, "[WS] Client error");
      clients.delete(ws);
    });

    // Handle ping-pong
    ws.on("message", (data) => {
      try {
        const msg = JSON.parse(data.toString());
        if (msg.type === "ping") {
          sendToClient(ws, {
            type: "pong",
            payload: {},
            timestamp: Date.now(),
          });
        }
      } catch {
        // ignore
      }
    });
  });

  // Start streaming Stellar ledger events with reconnection
  startLedgerStream();

  // Clean up on server close
  wss.on("close", () => {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    if (stopStream) {
      stopStream();
      stopStream = null;
    }
  });

  return wss;
}

function sendToClient(ws: WebSocket, event: PulsarEvent): void {
  if (ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(event));
  }
}

export function broadcast(event: PulsarEvent): void {
  const msg = JSON.stringify(event);
  clients.forEach((ws) => {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(msg);
    }
  });
}

/**
 * Broadcast a campaign event
 */
export function broadcastCampaignEvent(
  type: "campaign_created" | "view_recorded" | "payment_processed",
  data: Record<string, any>,
): void {
  broadcast({ type, payload: data, timestamp: Date.now() });
}

/**
 * Broadcast an auction event
 */
export function broadcastAuctionEvent(
  type: "bid_placed" | "auction_created" | "auction_settled",
  data: Record<string, any>,
): void {
  broadcast({ type, payload: data, timestamp: Date.now() });
}
