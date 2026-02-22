"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

export type TransactionStatus = "pending" | "success" | "failed";

export type TransactionType =
  | "campaign_create"
  | "campaign_fund"
  | "bid_place"
  | "payout"
  | "governance_vote"
  | "publisher_register"
  | "subscription"
  | "other";

export interface Transaction {
  txHash: string;
  type: TransactionType;
  status: TransactionStatus;
  timestamp: number;
  description: string;
  result?: any;
  error?: string;
}

interface TransactionStore {
  transactions: Transaction[];
  addTransaction: (tx: Omit<Transaction, "timestamp">) => void;
  updateTransaction: (txHash: string, updates: Partial<Transaction>) => void;
  getTransaction: (txHash: string) => Transaction | undefined;
  getPendingTransactions: () => Transaction[];
  clearOldTransactions: (olderThanDays?: number) => void;
}

export const useTransactionStore = create<TransactionStore>()(
  persist(
    (set, get) => ({
      transactions: [],

      addTransaction: (tx) =>
        set((state) => ({
          transactions: [
            {
              ...tx,
              timestamp: Date.now(),
            },
            ...state.transactions,
          ],
        })),

      updateTransaction: (txHash, updates) =>
        set((state) => ({
          transactions: state.transactions.map((tx) =>
            tx.txHash === txHash ? { ...tx, ...updates } : tx,
          ),
        })),

      getTransaction: (txHash) => {
        return get().transactions.find((tx) => tx.txHash === txHash);
      },

      getPendingTransactions: () => {
        return get().transactions.filter((tx) => tx.status === "pending");
      },

      clearOldTransactions: (olderThanDays = 30) => {
        const cutoffTime = Date.now() - olderThanDays * 24 * 60 * 60 * 1000;
        set((state) => ({
          transactions: state.transactions.filter(
            (tx) => tx.timestamp > cutoffTime || tx.status === "pending",
          ),
        }));
      },
    }),
    {
      name: "pulsar-tx-storage",
      storage: createJSONStorage(() =>
        typeof window !== "undefined"
          ? localStorage
          : ({
              getItem: () => null,
              setItem: () => {},
              removeItem: () => {},
            } as any),
      ),
    },
  ),
);
