"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

interface WalletStore {
  address: string | null;
  isConnected: boolean;
  network: string;
  networkMismatch: boolean;
  setAddress: (address: string | null) => void;
  setConnected: (connected: boolean) => void;
  setNetwork: (network: string) => void;
  setNetworkMismatch: (mismatch: boolean) => void;
  disconnect: () => void;
}

export const useWalletStore = create<WalletStore>()(
  persist(
    (set) => ({
      address: null,
      isConnected: false,
      network: "testnet",
      networkMismatch: false,
      setAddress: (address) => set({ address }),
      setConnected: (connected) => set({ isConnected: connected }),
      setNetwork: (network) => set({ network }),
      setNetworkMismatch: (networkMismatch) => set({ networkMismatch }),
      disconnect: () =>
        set({ address: null, isConnected: false, networkMismatch: false }),
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
