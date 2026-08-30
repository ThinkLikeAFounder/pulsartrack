import { describe, it, expect, vi, beforeEach } from 'vitest';

const { mockGetTransaction, mockUpdateTransaction } = vi.hoisted(() => ({
  mockGetTransaction: vi.fn(),
  mockUpdateTransaction: vi.fn(),
}));

vi.mock('./soroban-client', () => ({
  getSorobanServer: vi.fn().mockResolvedValue({ getTransaction: mockGetTransaction }),
}));

vi.mock('../store/tx-store', () => ({
  useTransactionStore: {
    getState: vi.fn().mockReturnValue({
      transactions: [],
      updateTransaction: mockUpdateTransaction,
    }),
  },
}));

vi.mock('@stellar/stellar-sdk', () => ({
  rpc: {
    Api: {
      GetTransactionStatus: {
        SUCCESS: 'SUCCESS',
        FAILED: 'FAILED',
        NOT_FOUND: 'NOT_FOUND',
      },
    },
  },
}));

import { checkPendingTransactions, pollTransaction } from './tx-recovery';
import { useTransactionStore } from '../store/tx-store';

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useTransactionStore.getState).mockReturnValue({
    transactions: [],
    updateTransaction: mockUpdateTransaction,
  } as any);
});

describe('checkPendingTransactions', () => {
  it('does nothing when no pending transactions', async () => {
    vi.mocked(useTransactionStore.getState).mockReturnValue({
      transactions: [{ txHash: 'h1', status: 'success' }],
      updateTransaction: mockUpdateTransaction,
    } as any);

    await checkPendingTransactions();
    expect(mockGetTransaction).not.toHaveBeenCalled();
  });

  it('marks transaction as success when confirmed', async () => {
    vi.mocked(useTransactionStore.getState).mockReturnValue({
      transactions: [{ txHash: 'h1', status: 'pending', timestamp: Date.now(), description: 'test' }],
      updateTransaction: mockUpdateTransaction,
    } as any);

    mockGetTransaction.mockResolvedValue({
      status: 'SUCCESS',
      returnValue: 'result_val',
    });

    await checkPendingTransactions();

    expect(mockUpdateTransaction).toHaveBeenCalledWith('h1', {
      status: 'success',
      result: 'result_val',
    });
  });

  it('marks transaction as failed when on-chain fails', async () => {
    vi.mocked(useTransactionStore.getState).mockReturnValue({
      transactions: [{ txHash: 'h2', status: 'pending', timestamp: Date.now(), description: 'test' }],
      updateTransaction: mockUpdateTransaction,
    } as any);

    mockGetTransaction.mockResolvedValue({ status: 'FAILED' });

    await checkPendingTransactions();

    expect(mockUpdateTransaction).toHaveBeenCalledWith('h2', {
      status: 'failed',
      error: 'Transaction failed on-chain',
    });
  });

  it('marks old not-found transaction as failed', async () => {
    const oneDayAgo = Date.now() - 25 * 60 * 60 * 1000;
    vi.mocked(useTransactionStore.getState).mockReturnValue({
      transactions: [{ txHash: 'h3', status: 'timeout', timestamp: oneDayAgo, description: 'test' }],
      updateTransaction: mockUpdateTransaction,
    } as any);

    mockGetTransaction.mockResolvedValue({ status: 'NOT_FOUND' });

    await checkPendingTransactions();

    expect(mockUpdateTransaction).toHaveBeenCalledWith('h3', {
      status: 'failed',
      error: 'Transaction not found (may have expired)',
    });
  });

  it('handles RPC errors gracefully', async () => {
    vi.mocked(useTransactionStore.getState).mockReturnValue({
      transactions: [{ txHash: 'h4', status: 'pending', timestamp: Date.now(), description: 'test' }],
      updateTransaction: mockUpdateTransaction,
    } as any);

    mockGetTransaction.mockRejectedValue(new Error('network error'));

    await checkPendingTransactions();

    expect(mockUpdateTransaction).not.toHaveBeenCalled();
  });
});

describe('pollTransaction', () => {
  it('returns success when transaction confirmed', async () => {
    mockGetTransaction.mockResolvedValue({
      status: 'SUCCESS',
      returnValue: 'ret',
    });

    const result = await pollTransaction('tx1', 2, 10);
    expect(result.success).toBe(true);
    expect(result.result).toBe('ret');
  });

  it('returns failure when transaction fails', async () => {
    mockGetTransaction.mockResolvedValue({ status: 'FAILED' });

    const result = await pollTransaction('tx2', 2, 10);
    expect(result.success).toBe(false);
    expect(result.error).toBe('Transaction failed on-chain');
  });

  it('returns timeout after max attempts', async () => {
    mockGetTransaction.mockResolvedValue({ status: 'NOT_FOUND' });

    const result = await pollTransaction('tx3', 2, 10);
    expect(result.success).toBe(false);
    expect(result.error).toBe('Polling timeout');
    expect(mockUpdateTransaction).toHaveBeenCalledWith('tx3', {
      status: 'timeout',
      error: 'Transaction confirmation timed out — check explorer',
    });
  });
});
