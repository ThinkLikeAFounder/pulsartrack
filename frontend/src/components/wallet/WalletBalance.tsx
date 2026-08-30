'use client';

import { useCallback, useEffect, useState } from 'react';
import { getHorizonUrl } from '../../lib/stellar-config';
import { useWalletStore } from '../../store/wallet-store';
import { TrendingUp, RefreshCw } from 'lucide-react';

export function WalletBalance() {
  const { address, isConnected } = useWalletStore();
  const [xlmBalance, setXlmBalance] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchBalance = useCallback(async (signal?: AbortSignal) => {
    if (!address) return;
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${getHorizonUrl()}/accounts/${address}`, { signal });
      if (res.status === 404) {
        setXlmBalance('0.00');
        setError('Account not funded');
        return;
      }
      if (!res.ok) throw new Error('Failed to load balance');
      const data: { balances?: Array<{ asset_type: string; balance: string }> } = await res.json();
      const xlm = data.balances?.find((b) => b.asset_type === 'native');
      setXlmBalance(xlm ? parseFloat(xlm.balance).toFixed(2) : '0.00');
    } catch (e) {
      if (e instanceof DOMException && e.name === 'AbortError') return;
      setXlmBalance(null);
      setError('Failed to load balance');
    } finally {
      setLoading(false);
    }
  }, [address]);

  useEffect(() => {
    if (isConnected && address) {
      const controller = new AbortController();
      fetchBalance(controller.signal);
      return () => controller.abort();
    }
  }, [isConnected, address, fetchBalance]);

  if (!isConnected || !address) return null;

  return (
    <div className="flex items-center gap-2 px-3 py-2 bg-blue-50 border border-blue-200 rounded-lg">
      <TrendingUp className="w-4 h-4 text-blue-600" />
      <div className="flex flex-col">
        <span className="text-sm font-medium text-blue-800">
          {loading ? '...' : xlmBalance !== null ? `${xlmBalance} XLM` : 'Balance unavailable'}
        </span>
        {error && (
          <span className="text-xs text-blue-500">{error}</span>
        )}
      </div>
      <button
        onClick={() => fetchBalance()}
        className="p-0.5 hover:text-blue-600 transition-colors"
        aria-label="Refresh balance"
        title="Refresh balance"
      >
        <RefreshCw className={`w-3 h-3 ${loading ? 'animate-spin' : ''}`} />
      </button>
    </div>
  );
}
