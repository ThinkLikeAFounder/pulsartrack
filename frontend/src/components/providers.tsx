'use client';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useState, useEffect } from 'react';
import { useWallet } from '../hooks/useWallet';
import { ContractProvider } from '@/contexts/ContractContext';
import { ToastProvider } from '@/contexts/ToastContext';
import { ThemeProvider } from './theme-provider';
import { useWalletStore } from '@/store/wallet-store';
import { connectWebSocket, disconnectWebSocket } from '@/lib/websocket';

import { ErrorBoundary } from './ErrorBoundary';

function WalletAutoReconnect({ children }: { children: React.ReactNode }) {
  const { checkConnection } = useWallet();

  useEffect(() => {
    checkConnection();
  }, [checkConnection]);

  return <>{children}</>;
}

/**
 * Owns the shared real-time connection: opens it when a wallet connects and
 * closes it on wallet disconnect / unmount. Pages only subscribe via
 * useAuctionEvents / useCampaignEvents.
 */
function WebSocketLifecycle() {
  const isConnected = useWalletStore((s) => s.isConnected);

  useEffect(() => {
    if (!isConnected) return;
    connectWebSocket();
    return () => disconnectWebSocket();
  }, [isConnected]);

  return null;
}

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 60 * 1000,
            refetchOnWindowFocus: false,
          },
        },
      })
  );

  return (
    <ErrorBoundary name="GlobalProviders">
      <QueryClientProvider client={queryClient}>
        <ThemeProvider>
          <ContractProvider>
            <ToastProvider>
              <WebSocketLifecycle />
              <WalletAutoReconnect>{children}</WalletAutoReconnect>
            </ToastProvider>
          </ContractProvider>
        </ThemeProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
