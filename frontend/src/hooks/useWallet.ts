"use client";

import { useEffect, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useWalletStore } from "../store/wallet-store";
import {
  connectWallet,
  isWalletConnected,
  getWalletAddress,
  verifyNetwork,
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
    disconnect: storeDisconnect,
  } = useWalletStore();
  const queryClient = useQueryClient();

  const connect = useCallback(async () => {
    try {
      const addr = await connectWallet();
      setAddress(addr);
      setConnected(true);
      return { success: true, address: addr };
    } catch (err) {
      const parsed = parseStellarError(err);
      console.error("Wallet connect error:", parsed);
      return { success: false, error: parsed };
    }
  }, [setAddress, setConnected]);

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
