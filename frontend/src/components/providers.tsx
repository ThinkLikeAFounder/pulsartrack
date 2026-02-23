'use client';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useState, useEffect } from 'react';
import { useWallet } from '../hooks/useWallet';
import { ContractProvider } from '@/contexts/ContractContext';
import { ToastProvider } from '@/contexts/ToastContext';

import { ErrorBoundary } from './ErrorBoundary';

function WalletAutoReconnect({ children }: { children: React.ReactNode }) {
  const { checkConnection } = useWallet();

  useEffect(() => {
    checkConnection();
  }, [checkConnection]);

  return <>{children}</>;
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
        <ContractProvider>
          <ToastProvider>
            <WalletAutoReconnect>{children}</WalletAutoReconnect>
          </ToastProvider>
        </ContractProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
