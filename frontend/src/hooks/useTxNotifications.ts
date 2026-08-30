"use client";

import { useEffect, useRef } from "react";
import { useTransactionStore } from "../store/tx-store";
import { useToast } from "../contexts/ToastContext";

/**
 * Hook to show notifications when transactions complete
 * Tracks transaction status changes and shows toast notifications
 */
export function useTxNotifications() {
  const { transactions } = useTransactionStore();
  const { success, error } = useToast();
  const previousTxsRef = useRef<Map<string, string>>(new Map());

  useEffect(() => {
    transactions.forEach((tx) => {
      const previousStatus = previousTxsRef.current.get(tx.txHash);

      // Transaction status changed from pending to success/failed
      if (previousStatus === "pending" && tx.status !== "pending") {
        if (tx.status === "success") {
          success("Transaction completed", tx.description);
          showBrowserNotification("success", tx.description);
        } else if (tx.status === "failed") {
          error("Transaction failed", tx.description);
          showBrowserNotification("error", tx.description);
        }
      }

      // Update the reference
      previousTxsRef.current.set(tx.txHash, tx.status);
    });
  }, [transactions, success, error]);
}

function showBrowserNotification(type: "success" | "error", message: string) {
  if ("Notification" in window && Notification.permission === "granted") {
    new Notification(
      type === "success" ? "Transaction Success" : "Transaction Failed",
      {
        body: message,
        icon: "/favicon.ico",
      },
    );
  }
}

/**
 * Request notification permission
 */
export function requestNotificationPermission() {
  if ("Notification" in window && Notification.permission === "default") {
    Notification.requestPermission();
  }
}
