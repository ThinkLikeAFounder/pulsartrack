import { Horizon } from '@stellar/stellar-sdk';
import { getHorizonServer, STELLAR_REQUEST_TIMEOUT_MS } from '../config/stellar';
import { logger } from '../lib/logger';

export function createHorizonServer(): Horizon.Server {
  const server = getHorizonServer();
  server.httpClient.defaults.timeout = STELLAR_REQUEST_TIMEOUT_MS;
  return server;
}

/**
 * Fetch account details from Horizon
 */
export async function getAccountDetails(address: string) {
  const server = createHorizonServer();
  try {
    const account = await server.loadAccount(address);
    const xlmBalance = account.balances.find((b: any) => b.asset_type === 'native');
    return {
      address,
      sequenceNumber: account.sequence,
      xlmBalance: xlmBalance ? parseFloat(xlmBalance.balance) : 0,
      balances: account.balances,
    };
  } catch (err: any) {
    if (err.response?.status === 404) {
      return null;
    }
    throw err;
  }
}

/**
 * Get recent transactions for an account
 */
export async function getAccountTransactions(
  address: string,
  limit = 20,
  cursor?: string,
  order: 'asc' | 'desc' = 'desc'
) {
  const server = createHorizonServer();
  let callBuilder = server
    .transactions()
    .forAccount(address)
    .limit(limit)
    .order(order);

  if (cursor) {
    callBuilder = callBuilder.cursor(cursor);
  }

  return callBuilder.call();
}

/**
 * Stream ledger events for contract activity with automatic reconnection
 */
export function streamLedgers(
  onLedger: (ledger: any) => void,
  onError?: (err: any) => void
): () => void {
  const server = createHorizonServer();
  let reconnectDelay = 1000;
  let isClosed = false;
  let es: any = null;

  function connect() {
    if (isClosed) return;

    es = server
      .ledgers()
      .cursor('now')
      .stream({
        onmessage: (ledger) => {
          reconnectDelay = 1000; // Reset on success
          onLedger(ledger);
        },
        onerror: (err) => {
          logger.error({ err }, '[Horizon] Stream error, reconnecting...');
          es?.close?.();
          
          if (onError) {
            onError(err);
          }

          if (!isClosed) {
            setTimeout(connect, reconnectDelay);
            reconnectDelay = Math.min(reconnectDelay * 2, 30000);
          }
        },
      });
  }

  connect();

  return () => {
    isClosed = true;
    es?.close?.();
  };
}

/**
 * Get Stellar network fee stats
 */
export async function getFeeStats() {
  const server = createHorizonServer();
  return server.feeStats();
}

/**
 * Get operations for a contract account
 */
export async function getContractOperations(contractId: string, limit = 50) {
  const server = createHorizonServer();
  try {
    const result = await server
      .operations()
      .forAccount(contractId)
      .limit(limit)
      .order('desc')
      .call();
    return result.records;
  } catch {
    return [];
  }
}

/**
 * Check if a Stellar address is funded (has minimum XLM reserve)
 */
export async function isAccountFunded(address: string): Promise<boolean> {
  const account = await getAccountDetails(address);
  return account !== null && account.xlmBalance >= 1;
}

export async function fetchFromHorizon(path: string) {
  const server = createHorizonServer();
  try {
    logger.debug({ path }, 'Fetching from Horizon');
    const response = await server.httpClient.get(path);
    return response.data;
  } catch (err) {
    logger.error({ err, path }, 'Horizon fetch failed');
    throw err;
  }
}

