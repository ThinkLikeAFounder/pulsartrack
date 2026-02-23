"use client";

import { useEffect, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useWalletStore } from "../store/wallet-store";
import {
  connectWallet,
  isWalletConnected,
  getWalletAddress,
  verifyNetwork,
  getFreighterNetworkLabel,
  getWalletData,
} from "../lib/wallet";
import { parseStellarError } from "../lib/error-handler";

export function useWallet() {
  const {
    address,
    isConnected,
    network,
    networkMismatch,
    setAddress,
    setConnected,
    setNetworkMismatch,
    setNetwork,
    setFreighterNetwork,
    disconnect: storeDisconnect,
    autoReconnect,
  } = useWalletStore();
  const queryClient = useQueryClient();

  const connect = useCallback(async () => {
    try {
      const addr = await connectWallet();

      // Update address/connected
      setAddress(addr);
      setConnected(true);

      // Immediately verify Freighter network after a successful connect
      const isNetworkCorrect = await verifyNetwork();
      const freighterLabel = await getFreighterNetworkLabel();
      setFreighterNetwork(freighterLabel || null);
      setNetworkMismatch(!isNetworkCorrect);

      // Ensure app network is set from wallet data (keeps store in sync)
      try {
        const data = await getWalletData();
        if (data.network) setNetwork(data.network);
      } catch {
        // ignore
      }

      return { success: true, address: addr };
    } catch (err) {
      const parsed = parseStellarError(err);
      console.error("Wallet connect error:", parsed);
      return { success: false, error: parsed };
    }
  }, [setAddress, setConnected, setFreighterNetwork, setNetworkMismatch, setNetwork]);

  const disconnect = useCallback(() => {
    // Freighter doesn't have a programmatic disconnect
    // Clear local state only
    storeDisconnect();
  }, [storeDisconnect]);

  const checkConnection = useCallback(async () => {
    const connected = await isWalletConnected();
    const isNetworkCorrect = await verifyNetwork();

    setNetworkMismatch(!isNetworkCorrect && connected);

    if (connected) {
      const addr = await getWalletAddress();
      if (addr && addr !== address) {
        setAddress(addr);
        setConnected(true);
        queryClient.invalidateQueries(); // Invalidate on account switch
        return;
      } else if (addr === address) {
        setConnected(true);
        return;
      }
    }

    if (isConnected && !connected) {
      storeDisconnect();
    }
  }, [
    address,
    isConnected,
    setAddress,
    setConnected,
    setNetworkMismatch,
    storeDisconnect,
    queryClient,
  ]);

  // Polling for wallet state changes
  useEffect(() => {
    const intervalId = setInterval(() => {
      checkConnection();
    }, 3000); // 3 seconds

    return () => clearInterval(intervalId);
  }, [checkConnection]);

  // Run autoReconnect on mount so store re-checks connection + network on rehydrate
  useEffect(() => {
    // ignore promise warning; autoReconnect is safe
    autoReconnect();
  }, [autoReconnect]);

  return {
    address,
    isConnected,
    network,
    networkMismatch,
    connect,
    disconnect,
    checkConnection,
  };
}
