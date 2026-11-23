'use client';

import { useState, useRef, useEffect, useCallback } from 'react';
import { Wallet, ExternalLink, AlertCircle } from 'lucide-react';
import { useWallet } from '../../hooks/useWallet';
import { formatAddress } from '../../lib/display-utils';
import { isDemoMode } from '../../lib/stellar-config';

interface WalletConnectButtonProps {
  className?: string;
}

export function WalletConnectButton({ className = '' }: WalletConnectButtonProps) {
  const { connect, isConnected, address } = useWallet();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showModal, setShowModal] = useState(false);
  const modalRef = useRef<HTMLDivElement>(null);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      setShowModal(false);
      return;
    }
    if (e.key !== 'Tab' || !modalRef.current) return;
    const focusable = modalRef.current.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }, []);

  useEffect(() => {
    if (showModal) {
      document.addEventListener('keydown', handleKeyDown);
      modalRef.current?.querySelector<HTMLElement>('button')?.focus();
    }
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [showModal, handleKeyDown]);

  const demoMode = isDemoMode();

  const handleConnect = async () => {
    // Demo mode generates a local address instantly — no need for the
    // "connecting..." dialog that real Freighter approval needs.
    if (!demoMode) setShowModal(true);
    setLoading(true);
    setError(null);
    const result = await connect();
    if (!result.success) {
      setError(
        demoMode
          ? 'Could not start the demo wallet. Please try again.'
          : 'Could not connect wallet. Please install Freighter and try again.',
      );
      setShowModal(true);
    } else {
      setShowModal(false);
    }
    setLoading(false);
  };

  if (isConnected && address) {
    return (
      <div className={`flex items-center gap-2 px-3 py-2 bg-green-50 border border-green-200 rounded-lg ${className}`}>
        <div className="w-2 h-2 bg-green-500 rounded-full" aria-hidden="true" />
        <span className="text-sm font-medium text-green-800">
          <span className="sr-only">Wallet connected: </span>
          {formatAddress(address)}
        </span>
      </div>
    );
  }

  return (
    <div className={className}>
      <button
        onClick={handleConnect}
        disabled={loading}
        aria-label={demoMode ? 'Connect demo wallet' : 'Connect Freighter wallet'}
        className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <Wallet className="w-4 h-4" aria-hidden="true" />
        {loading ? 'Connecting...' : demoMode ? 'Connect Demo Wallet' : 'Connect Freighter'}
      </button>
      {demoMode ? (
        <p className="mt-1 text-xs text-gray-500">
          Simulated wallet — no extension needed while contracts are in development.
        </p>
      ) : (
        <div className="mt-1 text-xs text-gray-500 flex items-center gap-1">
          <ExternalLink className="w-3 h-3" aria-hidden="true" />
          <a
            href="https://www.freighter.app/"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-blue-600"
          >
            Get Freighter wallet
          </a>
        </div>
      )}
      {showModal && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
          role="dialog"
          aria-modal="true"
          aria-label="Wallet connection"
        >
          <div ref={modalRef} className="bg-gray-800 border border-gray-700 rounded-xl p-6 max-w-sm w-full mx-4">
            {loading && (
              <p className="text-white text-sm text-center">
                {demoMode ? 'Connecting demo wallet...' : 'Connecting to Freighter...'}
              </p>
            )}
            {error && (
              <div className="space-y-3">
                <div className="flex items-center gap-1 text-sm text-red-400">
                  <AlertCircle className="w-4 h-4" aria-hidden="true" />
                  <span>{error}</span>
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={handleConnect}
                    className="flex-1 px-3 py-2 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 transition-colors"
                  >
                    Retry
                  </button>
                  <button
                    onClick={() => setShowModal(false)}
                    aria-label="Close wallet connection dialog"
                    className="px-3 py-2 border border-gray-600 text-gray-300 text-sm rounded-lg hover:bg-gray-700 transition-colors"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
