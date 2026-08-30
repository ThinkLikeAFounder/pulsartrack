"use client";

import type { rpc, xdr } from "@stellar/stellar-sdk";
import { useTransactionStore } from "../store/tx-store";

async function getSdk() {
  return import("@stellar/stellar-sdk");
}

async function getServer() {
  const { getSorobanServer } = await import("./soroban-client");
  return getSorobanServer();
}

/**
 * Check the status of pending transactions and update the store
 * Should be called on app initialization
 */
export async function checkPendingTransactions(): Promise<void> {
  const { transactions, updateTransaction } =
    useTransactionStore.getState();
  const pendingTxs = transactions.filter(
    (tx) => tx.status === "pending" || tx.status === "timeout",
  );

  if (pendingTxs.length === 0) return;

  const { rpc } = await getSdk();
  const server = await getServer();

  const checks = pendingTxs.map(async (tx) => {
    try {
      const result = await server.getTransaction(tx.txHash);

      if (result.status === rpc.Api.GetTransactionStatus.SUCCESS) {
        const returnVal = (result as rpc.Api.GetSuccessfulTransactionResponse)
          .returnValue;
        updateTransaction(tx.txHash, {
          status: "success",
          result: returnVal,
        });
      } else if (result.status === rpc.Api.GetTransactionStatus.FAILED) {
        updateTransaction(tx.txHash, {
          status: "failed",
          error: "Transaction failed on-chain",
        });
      } else if (result.status === rpc.Api.GetTransactionStatus.NOT_FOUND) {
        // Transaction might be too old or never made it to the ledger
        // Keep as pending for now, but could mark as failed after a certain time
        const ageInHours = (Date.now() - tx.timestamp) / (1000 * 60 * 60);
        if (ageInHours > 24) {
          updateTransaction(tx.txHash, {
            status: "failed",
            error: "Transaction not found (may have expired)",
          });
        }
      }
    } catch (error) {
      console.error(`Error checking transaction ${tx.txHash}:`, error);
    }
  });

  await Promise.allSettled(checks);
}

/**
 * Poll a specific transaction until it completes or times out
 */
export async function pollTransaction(
  txHash: string,
  maxAttempts: number = 10,
  initialIntervalMs: number = 2000,
): Promise<{ success: boolean; result?: xdr.ScVal; error?: string }> {
  const { rpc } = await getSdk();
  const server = await getServer();
  const { updateTransaction } = useTransactionStore.getState();

  for (let i = 0; i < maxAttempts; i++) {
    const delay = Math.min(initialIntervalMs * Math.pow(1.5, i), 10000);
    await new Promise((resolve) => setTimeout(resolve, delay));

    try {
      const result = await server.getTransaction(txHash);

      if (result.status === rpc.Api.GetTransactionStatus.SUCCESS) {
        const returnVal = (result as rpc.Api.GetSuccessfulTransactionResponse)
          .returnValue;
        updateTransaction(txHash, {
          status: "success",
          result: returnVal,
        });
        return { success: true, result: returnVal };
      }

      if (result.status === rpc.Api.GetTransactionStatus.FAILED) {
        updateTransaction(txHash, {
          status: "failed",
          error: "Transaction failed on-chain",
        });
        return { success: false, error: "Transaction failed on-chain" };
      }
    } catch (error) {
      console.error(`Error polling transaction ${txHash}:`, error);
    }
  }

  updateTransaction(txHash, {
    status: "timeout",
    error: "Transaction confirmation timed out — check explorer",
  });
  return { success: false, error: "Polling timeout" };
}
