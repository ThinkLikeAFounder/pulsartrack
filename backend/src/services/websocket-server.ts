import { WebSocketServer, WebSocket } from "ws";
import { IncomingMessage, Server } from "http";
import { streamLedgers } from "./horizon";
import { logger } from "../lib/logger";
import { decodeJwt } from "../lib/jwt";

interface PulsarEvent {
  type: string;
  payload: any;
  timestamp: number;
  txHash?: string;
  targetAccounts?: string[];
}

type ClientMessageType = "auth" | "subscribe" | "unsubscribe" | "ping";

interface ClientMessage {
  type: ClientMessageType;
  token?: string;
  channel?: string;
}

const VALID_CHANNELS = new Set(["ledger", "campaigns", "auctions"]);

interface ClientState {
  ws: WebSocket;
  subscriptions: Set<string>;
  stellarAddress: string;
}

const clients = new Map<WebSocket, ClientState>();

const connectionsPerIp = new Map<string, number>();
const MAX_CONNECTIONS_PER_IP = 5;

const AUTH_TIMEOUT_MS = 5000;

let stopStream: (() => void) | null = null;

function parseClientMessage(raw: string): ClientMessage | null {
  try {
    const msg = JSON.parse(raw);
    if (typeof msg !== "object" || msg === null) return null;
    if (!["auth", "subscribe", "unsubscribe", "ping"].includes(msg.type)) return null;
    if (msg.token !== undefined && typeof msg.token !== "string") return null;
    if (msg.channel !== undefined && typeof msg.channel !== "string") return null;
    return msg as ClientMessage;
  } catch {
    return null;
  }
}

function startLedgerStream(): void {
  stopStream = streamLedgers(
    (ledger) => {
      broadcastToChannel("ledger", {
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
      logger.error(err, "[WS] Ledger stream error notified");
      broadcastToChannel("ledger", {
        type: "reconnecting",
        payload: {
          message: "Horizon stream dropped, reconnecting...",
        },
        timestamp: Date.now(),
      });
    },
  );
}

export const MAX_PAYLOAD_SIZE = 16 * 1024;
export const MESSAGE_RATE_LIMIT_WINDOW_MS = 10000;
export const MAX_MESSAGES_PER_WINDOW = 30;

export function setupWebSocketServer(server: Server): WebSocketServer {
  const wss = new WebSocketServer({ server, path: "/ws", maxPayload: MAX_PAYLOAD_SIZE });

  wss.on("connection", (ws: WebSocket, req: IncomingMessage) => {
    const ip = (req.headers["x-forwarded-for"] as string)?.split(",")[0].trim()
      ?? req.socket.remoteAddress
      ?? "unknown";

    const ipCount = connectionsPerIp.get(ip) ?? 0;
    if (ipCount >= MAX_CONNECTIONS_PER_IP) {
      logger.warn(`[WS] Connection limit reached for IP ${ip}, rejecting`);
      ws.close(4029, "Too many connections");
      return;
    }
    connectionsPerIp.set(ip, ipCount + 1);

    sendToClient(ws, {
      type: "connected",
      payload: { message: "Connected to PulsarTrack WebSocket server. Send an auth message to authenticate." },
      timestamp: Date.now(),
    });

    let authenticated = false;
    let messageCount = 0;
    let resetTime = Date.now() + MESSAGE_RATE_LIMIT_WINDOW_MS;

    const authTimer = setTimeout(() => {
      if (!authenticated) {
        logger.warn(`[WS] Auth timeout for IP ${ip}`);
        ws.close(4001, "Authentication timeout");
        const remaining = (connectionsPerIp.get(ip) ?? 1) - 1;
        remaining > 0 ? connectionsPerIp.set(ip, remaining) : connectionsPerIp.delete(ip);
      }
    }, AUTH_TIMEOUT_MS);

    ws.on("close", () => {
      clearTimeout(authTimer);
      clients.delete(ws);
      const remaining = (connectionsPerIp.get(ip) ?? 1) - 1;
      remaining > 0 ? connectionsPerIp.set(ip, remaining) : connectionsPerIp.delete(ip);
      logger.info(`[WS] Client disconnected. Total: ${clients.size}`);
    });

    ws.on("error", (err) => {
      clearTimeout(authTimer);
      logger.error(err, "[WS] Client error");
      clients.delete(ws);
      const remaining = (connectionsPerIp.get(ip) ?? 1) - 1;
      remaining > 0 ? connectionsPerIp.set(ip, remaining) : connectionsPerIp.delete(ip);
    });

    ws.on("message", (data) => {
      const msg = parseClientMessage(data.toString());
      if (!msg) {
        sendToClient(ws, {
          type: "error",
          payload: { message: "Invalid message format" },
          timestamp: Date.now(),
        });
        return;
      }

      if (!authenticated) {
        if (msg.type === "auth") {
          let payload: Record<string, any>;
          try {
            if (!msg.token) throw new Error("Missing token in auth message");
            payload = decodeJwt(msg.token);
            if (typeof payload.sub !== "string" || !payload.sub) {
              throw new Error("Invalid token subject");
            }
          } catch (err) {
            clearTimeout(authTimer);
            logger.warn(`[WS] Auth failed from ${ip}: ${(err as Error).message}`);
            ws.close(4001, "Unauthorized");
            const remaining = (connectionsPerIp.get(ip) ?? 1) - 1;
            remaining > 0 ? connectionsPerIp.set(ip, remaining) : connectionsPerIp.delete(ip);
            return;
          }

          clearTimeout(authTimer);
          authenticated = true;
          const state: ClientState = {
            ws,
            subscriptions: new Set(),
            stellarAddress: payload.sub,
          };
          clients.set(ws, state);
          logger.info(`[WS] Client authenticated (${payload.sub}). Total: ${clients.size}`);

          sendToClient(ws, {
            type: "authenticated",
            payload: { address: payload.sub },
            timestamp: Date.now(),
          });
          return;
        }

        sendToClient(ws, {
          type: "error",
          payload: { message: "Not authenticated. Send an auth message first." },
          timestamp: Date.now(),
        });
        return;
      }

      const state = clients.get(ws)!;

      if (Date.now() > resetTime) {
        messageCount = 0;
        resetTime = Date.now() + MESSAGE_RATE_LIMIT_WINDOW_MS;
      }
      messageCount++;
      if (messageCount > MAX_MESSAGES_PER_WINDOW) {
        sendToClient(ws, {
          type: "error",
          payload: { message: "Rate limit exceeded" },
          timestamp: Date.now(),
        });
        return;
      }

      if (msg.type === "ping") {
        sendToClient(ws, { type: "pong", payload: {}, timestamp: Date.now() });
        return;
      }

      const channel = msg.channel ?? "";
      if (!VALID_CHANNELS.has(channel)) {
        sendToClient(ws, {
          type: "error",
          payload: { message: `Unknown channel: ${channel}` },
          timestamp: Date.now(),
        });
        return;
      }

      if (msg.type === "subscribe") {
        state.subscriptions.add(channel);
        sendToClient(ws, {
          type: "subscribed",
          payload: { channel },
          timestamp: Date.now(),
        });
      } else if (msg.type === "unsubscribe") {
        state.subscriptions.delete(channel);
        sendToClient(ws, {
          type: "unsubscribed",
          payload: { channel },
          timestamp: Date.now(),
        });
      }
    });
  });

  startLedgerStream();

  wss.on("close", () => {
    if (stopStream) { stopStream(); stopStream = null; }
  });

  return wss;
}

function sendToClient(ws: WebSocket, event: PulsarEvent): void {
  if (ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(event));
  }
}

export function broadcastToChannel(channel: string, event: PulsarEvent): void {
  const msg = JSON.stringify(event);
  clients.forEach((state) => {
    const isTargeted =
      Array.isArray(event.targetAccounts) && event.targetAccounts.length > 0;
    const allowedForClient =
      !isTargeted || event.targetAccounts?.includes(state.stellarAddress) === true;

    if (
      allowedForClient
      && state.subscriptions.has(channel)
      && state.ws.readyState === WebSocket.OPEN
    ) {
      state.ws.send(msg);
    }
  });
}

export function broadcast(event: PulsarEvent): void {
  const msg = JSON.stringify(event);
  clients.forEach((state) => {
    const isTargeted =
      Array.isArray(event.targetAccounts) && event.targetAccounts.length > 0;
    const allowedForClient =
      !isTargeted || event.targetAccounts?.includes(state.stellarAddress) === true;

    if (allowedForClient && state.ws.readyState === WebSocket.OPEN) {
      state.ws.send(msg);
    }
  });
}

export function broadcastCampaignEvent(
  type: "campaign_created" | "view_recorded" | "payment_processed",
  data: Record<string, any>,
): void {
  broadcastToChannel("campaigns", { type, payload: data, timestamp: Date.now() });
}

export function broadcastAuctionEvent(
  type: "bid_placed" | "auction_created" | "auction_settled",
  data: Record<string, any>,
): void {
  broadcastToChannel("auctions", { type, payload: data, timestamp: Date.now() });
}
