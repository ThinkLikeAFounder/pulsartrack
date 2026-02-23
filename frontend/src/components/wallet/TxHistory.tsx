"use client";

import { useEffect, useState } from "react";
import {
  X,
  Clock,
  CheckCircle,
  XCircle,
  ExternalLink,
  RefreshCw,
} from "lucide-react";
import { useTransactionStore, Transaction } from "../../store/tx-store";
import { pollTransaction } from "../../lib/tx-recovery";

interface TxHistoryProps {
  isOpen: boolean;
  onClose: () => void;
}

export function TxHistory({ isOpen, onClose }: TxHistoryProps) {
  const { transactions, clearOldTransactions } = useTransactionStore();
  const [pollingTxHash, setPollingTxHash] = useState<string | null>(null);

  useEffect(() => {
    // Clean up old transactions when component mounts
    clearOldTransactions(30);
  }, [clearOldTransactions]);

  const handleRetryPoll = async (txHash: string) => {
    setPollingTxHash(txHash);
    await pollTransaction(txHash, 5, 2000);
    setPollingTxHash(null);
  };

  const getStatusIcon = (status: Transaction["status"]) => {
    switch (status) {
      case "pending":
        return <Clock className="w-5 h-5 text-yellow-500 animate-pulse" />;
      case "success":
        return <CheckCircle className="w-5 h-5 text-green-500" />;
      case "failed":
        return <XCircle className="w-5 h-5 text-red-500" />;
    }
  };

  const getStatusText = (status: Transaction["status"]) => {
    switch (status) {
      case "pending":
        return "Pending";
      case "success":
        return "Success";
      case "failed":
        return "Failed";
    }
  };

  const getTypeLabel = (type: Transaction["type"]) => {
    const labels: Record<Transaction["type"], string> = {
      campaign_create: "Create Campaign",
      campaign_fund: "Fund Campaign",
      bid_place: "Place Bid",
      payout: "Payout",
      governance_vote: "Governance Vote",
      publisher_register: "Register Publisher",
      subscription: "Subscription",
      other: "Transaction",
    };
    return labels[type];
  };

  const formatTimestamp = (timestamp: number) => {
    const date = new Date(timestamp);
    const now = Date.now();
    const diff = now - timestamp;

    // Less than 1 minute
    if (diff < 60000) return "Just now";
    // Less than 1 hour
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    // Less than 1 day
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    // Less than 7 days
    if (diff < 604800000) return `${Math.floor(diff / 86400000)}d ago`;

    return date.toLocaleDateString();
  };

  const getExplorerUrl = (txHash: string) => {
    // Adjust based on network (testnet/mainnet)
    return `https://stellar.expert/explorer/testnet/tx/${txHash}`;
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 overflow-hidden">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Drawer */}
      <div className="absolute right-0 top-0 h-full w-full max-w-md bg-white shadow-2xl">
        <div className="flex h-full flex-col">
          {/* Header */}
          <div className="flex items-center justify-between border-b border-gray-200 px-6 py-4">
            <h2 className="text-xl font-semibold text-gray-900">
              Transaction History
            </h2>
            <button
              onClick={onClose}
              className="rounded-lg p-2 hover:bg-gray-100 transition-colors"
              aria-label="Close"
            >
              <X className="w-5 h-5 text-gray-500" />
            </button>
          </div>

          {/* Transaction List */}
          <div className="flex-1 overflow-y-auto px-6 py-4">
            {transactions.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-full text-center">
                <Clock className="w-12 h-12 text-gray-300 mb-3" />
                <p className="text-gray-500">No transactions yet</p>
                <p className="text-sm text-gray-400 mt-1">
                  Your transaction history will appear here
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                {transactions.map((tx) => (
                  <div
                    key={tx.txHash}
                    className="rounded-lg border border-gray-200 bg-white p-4 hover:shadow-md transition-shadow"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-start gap-3 flex-1">
                        {getStatusIcon(tx.status)}
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <p className="font-medium text-gray-900">
                              {getTypeLabel(tx.type)}
                            </p>
                            <span
                              className={`text-xs px-2 py-0.5 rounded-full ${
                                tx.status === "pending"
                                  ? "bg-yellow-100 text-yellow-700"
                                  : tx.status === "success"
                                    ? "bg-green-100 text-green-700"
                                    : "bg-red-100 text-red-700"
                              }`}
                            >
                              {getStatusText(tx.status)}
                            </span>
                          </div>
                          <p className="text-sm text-gray-600 mt-1">
                            {tx.description}
                          </p>
                          <p className="text-xs text-gray-400 mt-1">
                            {formatTimestamp(tx.timestamp)}
                          </p>
                          {tx.error && (
                            <p className="text-xs text-red-600 mt-1">
                              Error: {tx.error}
                            </p>
                          )}
                        </div>
                      </div>
                    </div>

                    {/* Actions */}
                    <div className="flex items-center gap-2 mt-3 pt-3 border-t border-gray-100">
                      <a
                        href={getExplorerUrl(tx.txHash)}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="flex items-center gap-1 text-xs text-indigo-600 hover:text-indigo-700 transition-colors"
                      >
                        <ExternalLink className="w-3 h-3" />
                        View on Explorer
                      </a>
                      {tx.status === "pending" && (
                        <button
                          onClick={() => handleRetryPoll(tx.txHash)}
                          disabled={pollingTxHash === tx.txHash}
                          className="flex items-center gap-1 text-xs text-gray-600 hover:text-gray-700 transition-colors disabled:opacity-50"
                        >
                          <RefreshCw
                            className={`w-3 h-3 ${
                              pollingTxHash === tx.txHash ? "animate-spin" : ""
                            }`}
                          />
                          Check Status
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
