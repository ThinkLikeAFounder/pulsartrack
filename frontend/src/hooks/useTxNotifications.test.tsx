import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import type { Transaction } from '../store/tx-store';

const storeState = vi.hoisted(() => ({
  transactions: [] as Transaction[],
}));

vi.mock('../store/tx-store', () => ({
  useTransactionStore: () => ({ transactions: storeState.transactions }),
}));

const { mockSuccess, mockError } = vi.hoisted(() => ({
  mockSuccess: vi.fn(),
  mockError: vi.fn(),
}));

vi.mock('../contexts/ToastContext', () => ({
  useToast: vi.fn(() => ({
    success: mockSuccess,
    error: mockError,
    warning: vi.fn(),
    info: vi.fn(),
    dismiss: vi.fn(),
    toasts: [],
  })),
}));

import { useTxNotifications } from './useTxNotifications';
import { useToast } from '../contexts/ToastContext';

function transaction(txHash: string, status: Transaction['status'], description: string): Transaction {
  return {
    txHash,
    status,
    description,
    type: 'other',
    timestamp: Date.now(),
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  storeState.transactions = [];
});

describe('useTxNotifications', () => {
  it('calls useToast on mount', () => {
    renderHook(() => useTxNotifications());
    expect(useToast).toHaveBeenCalled();
  });

  it('shows success toast on pending to success transition', () => {
    storeState.transactions = [transaction('h1', 'pending', 'Lock')];
    const { rerender } = renderHook(() => useTxNotifications());

    storeState.transactions = [transaction('h1', 'success', 'Lock')];
    rerender();

    expect(mockSuccess).toHaveBeenCalledWith('Transaction completed', 'Lock');
  });

  it('shows error toast on pending to failed transition', () => {
    storeState.transactions = [transaction('h2', 'pending', 'Unlock')];
    const { rerender } = renderHook(() => useTxNotifications());

    storeState.transactions = [transaction('h2', 'failed', 'Unlock')];
    rerender();

    expect(mockError).toHaveBeenCalledWith('Transaction failed', 'Unlock');
  });

  it('does not notify for pending-only transactions', () => {
    storeState.transactions = [transaction('h3', 'pending', 'Vote')];
    renderHook(() => useTxNotifications());

    expect(mockSuccess).not.toHaveBeenCalled();
    expect(mockError).not.toHaveBeenCalled();
  });
});