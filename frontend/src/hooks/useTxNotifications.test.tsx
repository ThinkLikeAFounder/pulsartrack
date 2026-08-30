import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import React from 'react';

// Shared mutable state for the mock store
let currentTransactions: any[] = [];

vi.mock('../store/tx-store', () => ({
  useTransactionStore: (() => ({ transactions: currentTransactions })) as any,
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

beforeEach(() => {
  vi.clearAllMocks();
  currentTransactions = [];
});

describe('useTxNotifications', () => {
  it('calls useToast on mount', () => {
    renderHook(() => useTxNotifications());
    expect(useToast).toHaveBeenCalled();
  });

  it('shows success toast on pending→success transition', () => {
    // Start with pending transaction
    currentTransactions = [
      { txHash: 'h1', status: 'pending', description: 'Lock', timestamp: Date.now() },
    ];
    const { rerender } = renderHook(() => useTxNotifications());

    // Transition to success — must use new array reference for React to detect change
    currentTransactions = [
      { txHash: 'h1', status: 'success', description: 'Lock', timestamp: Date.now() },
    ];
    rerender();

    expect(mockSuccess).toHaveBeenCalledWith('Transaction completed', 'Lock');
  });

  it('shows error toast on pending→failed transition', () => {
    currentTransactions = [
      { txHash: 'h2', status: 'pending', description: 'Unlock', timestamp: Date.now() },
    ];
    const { rerender } = renderHook(() => useTxNotifications());

    currentTransactions = [
      { txHash: 'h2', status: 'failed', description: 'Unlock', timestamp: Date.now() },
    ];
    rerender();

    expect(mockError).toHaveBeenCalledWith('Transaction failed', 'Unlock');
  });

  it('does not notify for pending-only transactions', () => {
    currentTransactions = [
      { txHash: 'h3', status: 'pending', description: 'Vote', timestamp: Date.now() },
    ];
    renderHook(() => useTxNotifications());

    expect(mockSuccess).not.toHaveBeenCalled();
    expect(mockError).not.toHaveBeenCalled();
  });
});
