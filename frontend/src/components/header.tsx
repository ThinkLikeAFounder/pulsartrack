"use client";

import { useEffect, useState } from "react";
import { Menu, X, Zap, History, AlertTriangle } from "lucide-react";
import { useWalletStore } from "../store/wallet-store";
import { useTransactionStore } from "../store/tx-store";
import { WalletConnectButton } from "./wallet/WalletModal";
import { AccountSwitcher } from "./wallet/AccountSwitcher";
import { WalletBalance } from "./wallet/WalletBalance";
import { TxHistory } from "./wallet/TxHistory";
import { checkPendingTransactions } from "../lib/tx-recovery";
import { useTxNotifications } from "../hooks/useTxNotifications";

export function Header() {
  const { address, isConnected } = useWalletStore();
  const { getPendingTransactions } = useTransactionStore();
  const [mounted, setMounted] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [txHistoryOpen, setTxHistoryOpen] = useState(false);
  const [pendingCount, setPendingCount] = useState(0);

  // Enable transaction notifications
  useTxNotifications();

  useEffect(() => {
    setMounted(true);

    // Check pending transactions on mount
    checkPendingTransactions();

    // Set up interval to check pending transactions periodically
    const interval = setInterval(() => {
      checkPendingTransactions();
    }, 30000); // Check every 30 seconds

    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    // Update pending count
    if (mounted) {
      setPendingCount(getPendingTransactions().length);
    }
  }, [mounted, getPendingTransactions]);

  if (!mounted) return null;

  return (
    <>
      {isConnected && useWalletStore.getState().networkMismatch && (
        <div className="bg-amber-500 text-white px-4 py-2 text-sm font-medium text-center flex items-center justify-center gap-2">
          <AlertTriangle className="w-4 h-4" />
          <span>
            Please switch your Freighter wallet to{" "}
            {process.env.NEXT_PUBLIC_STELLAR_NETWORK || "testnet"}
          </span>
        </div>
      )}
      <header className="border-b border-gray-200 bg-white sticky top-0 z-40 backdrop-blur-sm bg-white/80">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="flex h-16 items-center justify-between">
            {/* Logo */}
            <div className="flex items-center gap-8">
              <a href="/" className="flex items-center gap-2">
                <div className="w-8 h-8 bg-gradient-to-br from-indigo-600 to-cyan-500 rounded-lg flex items-center justify-center">
                  <Zap className="w-5 h-5 text-white" />
                </div>
                <h1 className="text-2xl font-bold text-gray-900">
                  PulsarTrack
                </h1>
              </a>

              {/* Desktop Navigation */}
              <nav className="hidden md:flex items-center space-x-6">
                <a
                  href="/"
                  className="text-gray-700 hover:text-indigo-600 transition-colors font-medium"
                >
                  Home
                </a>
                <a
                  href="/advertiser"
                  className="text-gray-700 hover:text-indigo-600 transition-colors font-medium"
                >
                  Advertiser
                </a>
                <a
                  href="/publisher"
                  className="text-gray-700 hover:text-indigo-600 transition-colors font-medium"
                >
                  Publisher
                </a>
                <a
                  href="/governance"
                  className="text-gray-700 hover:text-indigo-600 transition-colors font-medium"
                >
                  Governance
                </a>
              </nav>
            </div>

            {/* Desktop Wallet Connection */}
            <div className="hidden md:flex items-center gap-3">
              {isConnected && address ? (
                <>
                  <WalletBalance />
                  <button
                    onClick={() => setTxHistoryOpen(true)}
                    className="relative p-2 hover:bg-gray-100 rounded-lg transition-colors"
                    aria-label="Transaction history"
                  >
                    <History className="w-5 h-5 text-gray-700" />
                    {pendingCount > 0 && (
                      <span className="absolute -top-1 -right-1 w-5 h-5 bg-yellow-500 text-white text-xs rounded-full flex items-center justify-center font-medium">
                        {pendingCount}
                      </span>
                    )}
                  </button>
                  <AccountSwitcher />
                </>
              ) : (
                <WalletConnectButton />
              )}
            </div>

            {/* Mobile Menu Button */}
            <button
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              className="md:hidden p-2 hover:bg-gray-100 rounded-lg transition-colors"
              aria-label="Toggle menu"
            >
              {mobileMenuOpen ? (
                <X className="w-6 h-6 text-gray-700" />
              ) : (
                <Menu className="w-6 h-6 text-gray-700" />
              )}
            </button>
          </div>

          {/* Mobile Menu */}
          {mobileMenuOpen && (
            <div className="md:hidden border-t border-gray-200 py-4">
              <nav className="flex flex-col space-y-4">
                {[
                  { href: "/", label: "Home" },
                  { href: "/advertiser", label: "Advertiser" },
                  { href: "/publisher", label: "Publisher" },
                  { href: "/governance", label: "Governance" },
                ].map(({ href, label }) => (
                  <a
                    key={href}
                    href={href}
                    className="text-gray-700 hover:text-indigo-600 transition-colors font-medium px-2 py-1"
                    onClick={() => setMobileMenuOpen(false)}
                  >
                    {label}
                  </a>
                ))}

                <div className="pt-4 border-t border-gray-200">
                  {isConnected && address ? (
                    <>
                      <button
                        onClick={() => {
                          setTxHistoryOpen(true);
                          setMobileMenuOpen(false);
                        }}
                        className="w-full flex items-center justify-between px-2 py-2 text-gray-700 hover:text-indigo-600 transition-colors font-medium mb-2"
                      >
                        <span className="flex items-center gap-2">
                          <History className="w-5 h-5" />
                          Transaction History
                        </span>
                        {pendingCount > 0 && (
                          <span className="w-5 h-5 bg-yellow-500 text-white text-xs rounded-full flex items-center justify-center font-medium">
                            {pendingCount}
                          </span>
                        )}
                      </button>
                      <AccountSwitcher className="w-full" />
                    </>
                  ) : (
                    <WalletConnectButton />
                  )}
                </div>
              </nav>
            </div>
          )}
        </div>
      </header>

      {/* Transaction History Drawer */}
      <TxHistory
        isOpen={txHistoryOpen}
        onClose={() => setTxHistoryOpen(false)}
      />
    </>
  );
}
