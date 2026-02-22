"use client";

import { useEffect, useRef } from "react";
import { useTransactionStore } from "../store/tx-store";

/**
 * Hook to show notifications when transactions complete
 * Tracks transaction status changes and shows toast notifications
 */
export function useTxNotifications() {
  const { transactions } = useTransactionStore();
  const previousTxsRef = useRef<Map<string, string>>(new Map());

  useEffect(() => {
    transactions.forEach((tx) => {
      const previousStatus = previousTxsRef.current.get(tx.txHash);

      // Transaction status changed from pending to success/failed
      if (previousStatus === "pending" && tx.status !== "pending") {
        if (tx.status === "success") {
          showNotification(
            "success",
            `Transaction completed: ${tx.description}`,
          );
        } else if (tx.status === "failed") {
          showNotification("error", `Transaction failed: ${tx.description}`);
        }
      }

      // Update the reference
      previousTxsRef.current.set(tx.txHash, tx.status);
    });
  }, [transactions]);
}

function showNotification(type: "success" | "error", message: string) {
  // Use browser notification API if available
  if ("Notification" in window && Notification.permission === "granted") {
    new Notification(
      type === "success" ? "✅ Transaction Success" : "❌ Transaction Failed",
      {
        body: message,
        icon: "/favicon.ico",
      },
    );
  }

  // Also log to console for debugging
  console.log(`[TX ${type.toUpperCase()}]`, message);
}

/**
 * Request notification permission
 */
export function requestNotificationPermission() {
  if ("Notification" in window && Notification.permission === "default") {
    Notification.requestPermission();
  }
}
