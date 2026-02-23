import { Horizon } from '@stellar/stellar-sdk';
import { getHorizonServer } from '../config/stellar';
import { logger } from '../lib/logger';

/**
 * Fetch account details from Horizon
 */
export async function getAccountDetails(address: string) {
  const server = getHorizonServer();
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
export async function getAccountTransactions(address: string, limit = 20) {
  const server = getHorizonServer();
  const result = await server
    .transactions()
    .forAccount(address)
    .limit(limit)
    .order('desc')
    .call();
  return result.records;
}

/**
 * Stream ledger events for contract activity
 */
export function streamLedgers(
  onLedger: (ledger: any) => void,
  onError?: (err: any) => void
): () => void {
  const server = getHorizonServer();
  const es = server
    .ledgers()
    .cursor('now')
    .stream({
      onmessage: onLedger,
      onerror: onError,
    });

  return () => (es as any)?.close?.();
}

/**
 * Get Stellar network fee stats
 */
export async function getFeeStats() {
  const server = getHorizonServer();
  return server.feeStats();
}

/**
 * Get operations for a contract account
 */
export async function getContractOperations(contractId: string, limit = 50) {
  const server = getHorizonServer();
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

async function fetchFromHorizon(path: string) {
  try {
    logger.debug({ path }, 'Fetching from Horizon');
  } catch (err) {
    logger.error({ err, path }, 'Horizon fetch failed');
    throw err;
  }
}
