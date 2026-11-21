"use client";

import { AlertTriangle } from "lucide-react";
import { useWalletStore } from "../../store/wallet-store";

/**
 * Shown under the header while the connected Freighter wallet is on a
 * different network from the one this app is configured for.
 */
export default function NetworkMismatchBanner() {
  const { isConnected, networkMismatch, freighterNetwork, network } = useWalletStore();

  if (!isConnected || !networkMismatch) return null;

  return (
    <div
      role="alert"
      className="bg-amber-500 text-white px-4 py-2 text-sm font-medium text-center flex items-center justify-center gap-2"
    >
      <AlertTriangle className="w-4 h-4 shrink-0" aria-hidden="true" />
      <span>
        Network mismatch: your Freighter wallet is on {freighterNetwork || "an unknown network"}, but this app
        uses {network}. Switch networks in Freighter.
      </span>
    </div>
  );
}
