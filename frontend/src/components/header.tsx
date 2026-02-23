"use client";

import { useEffect, useState } from "react";
import { Menu, X, Zap, AlertTriangle } from "lucide-react";
import { useWalletStore } from "../store/wallet-store";
import { WalletConnectButton } from "./wallet/WalletModal";
import { AccountSwitcher } from "./wallet/AccountSwitcher";
import { WalletBalance } from "./wallet/WalletBalance";

export function Header() {
  const { address, isConnected } = useWalletStore();
  const [mounted, setMounted] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

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
                    <AccountSwitcher className="w-full" />
                  ) : (
                    <WalletConnectButton />
                  )}
                </div>
              </nav>
            </div>
          )}
        </div>
      </header>
    </>
  );
}
