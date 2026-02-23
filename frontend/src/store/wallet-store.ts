"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { isWalletConnected, getWalletAddress, verifyNetwork, getFreighterNetworkLabel } from "../lib/wallet";

interface WalletStore {
  address: string | null;
  isConnected: boolean;
  network: string;
  freighterNetwork: string | null;
  networkMismatch: boolean;
  setAddress: (address: string | null) => void;
  setConnected: (connected: boolean) => void;
  setNetwork: (network: string) => void;
  setFreighterNetwork: (network: string | null) => void;
  setNetworkMismatch: (mismatch: boolean) => void;
  disconnect: () => void;
  autoReconnect: () => Promise<void>;
}

export const useWalletStore = create<WalletStore>()(
  persist(
    (set, get) => ({
      address: null,
      isConnected: false,
      network: "testnet",
      freighterNetwork: null,
      networkMismatch: false,
      setAddress: (address) => set({ address }),
      setConnected: (connected) => set({ isConnected: connected }),
      setNetwork: (network) => set({ network }),
      setFreighterNetwork: (freighterNetwork) => set({ freighterNetwork }),
      setNetworkMismatch: (networkMismatch) => set({ networkMismatch }),
      disconnect: () =>
        set({ address: null, isConnected: false, networkMismatch: false, freighterNetwork: null }),
      // Auto-reconnect flow callable by client code (checks connection and network)
      autoReconnect: async () => {
        // Only operate on client; fail silently otherwise
        try {
          const connected = await isWalletConnected();
          if (!connected) {
            set({ address: null, isConnected: false, networkMismatch: false, freighterNetwork: null });
            return;
          }

          const addr = await getWalletAddress();
          const isNetworkCorrect = await verifyNetwork();
          const freighterLabel = await getFreighterNetworkLabel();

          set({
            address: addr,
            isConnected: !!addr,
            freighterNetwork: freighterLabel,
            networkMismatch: !isNetworkCorrect && !!connected,
          });
        } catch {
          // ignore errors during autoReconnect
        }
      },
    }),
    {
      name: "pulsar-wallet-storage",
      storage: createJSONStorage(() =>
        typeof window !== "undefined"
          ? localStorage
          : ({
              getItem: () => null,
              setItem: () => {},
              removeItem: () => {},
            } as any),
      ),
    },
  ),
);
