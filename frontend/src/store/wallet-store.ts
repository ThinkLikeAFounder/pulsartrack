"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import type { StateStorage } from "zustand/middleware";
import { isWalletConnected, getWalletAddress, verifyNetwork, getFreighterNetworkLabel } from "../lib/wallet";

interface WalletStore {
  address: string | null;
  isConnected: boolean;
  network: string;
  freighterNetwork: string | null;
  networkMismatch: boolean;
  /** Issue #368 — true once Zustand has rehydrated from localStorage. */
  _hydrated: boolean;
  setAddress: (address: string | null) => void;
  setConnected: (connected: boolean) => void;
  setNetwork: (network: string) => void;
  setFreighterNetwork: (network: string | null) => void;
  setNetworkMismatch: (mismatch: boolean) => void;
  disconnect: () => void;
  autoReconnect: () => Promise<void>;
  setHydrated: (value: boolean) => void;
}

export const useWalletStore = create<WalletStore>()(
  persist(
    (set) => ({
      address: null,
      isConnected: false,
      network: "testnet",
      freighterNetwork: null,
      networkMismatch: false,
      _hydrated: false,
      setAddress: (address) => set({ address }),
      setConnected: (connected) => set({ isConnected: connected }),
      setNetwork: (network) => set({ network }),
      setFreighterNetwork: (freighterNetwork) => set({ freighterNetwork }),
      setNetworkMismatch: (networkMismatch) => set({ networkMismatch }),
      setHydrated: (value) => set({ _hydrated: value }),
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
            isConnected: !!addr && isNetworkCorrect,
            freighterNetwork: freighterLabel,
            networkMismatch: !isNetworkCorrect && !!connected,
          });
        } catch (error) {
          // Issue #790 — don't silently swallow. A background reconnect failure
          // shouldn't interrupt the user with a toast, but it must leave a
          // diagnostic trail and not leave a half-connected state lingering.
          console.error("Wallet auto-reconnect failed:", error);
          set({ address: null, isConnected: false, networkMismatch: false, freighterNetwork: null });
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
            } satisfies StateStorage),
      ),
      // Issue #368 — set _hydrated=true once Zustand has read localStorage.
      // Components can gate rendering on this flag instead of using a local
      // `mounted` useState, which eliminates the header flicker.
      onRehydrateStorage: () => (state) => {
        state?.setHydrated(true);
      },
    },
  ),
);
