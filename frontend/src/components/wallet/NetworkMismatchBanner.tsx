"use client";

import React from "react";
import { useWalletStore } from "../../store/wallet-store";

/**
 * Persistent banner shown when Freighter network doesn't match app network.
 */
export default function NetworkMismatchBanner() {
  const { networkMismatch, freighterNetwork, network } = useWalletStore();

  if (!networkMismatch) return null;

  return (
    <div style={{
      position: "fixed",
      top: 0,
      left: 0,
      right: 0,
      background: "#ffecec",
      color: "#611a15",
      padding: "12px 16px",
      zIndex: 9999,
      borderBottom: "1px solid #f5c6cb",
      textAlign: "center",
    }}>
      <strong>Network mismatch:</strong>{" "}
      Your Freighter wallet is set to <em>{freighterNetwork || "unknown"}</em>. This app requires <em>{network}</em>. Please switch networks in the Freighter extension.
    </div>
  );
}
