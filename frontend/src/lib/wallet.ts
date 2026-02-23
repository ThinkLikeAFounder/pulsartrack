'use client';

import {
  requestAccess,
  isAllowed,
  getAddress,
  signTransaction,
  getNetworkDetails,
  isConnected as freighterIsConnected,
} from '@stellar/freighter-api';
import { CURRENT_NETWORK, getNetworkPassphrase } from './stellar-config';

export interface WalletData {
  address: string;
  isConnected: boolean;
  network: string;
}

/**
 * Check if Freighter wallet extension is available
 */
export const isFreighterAvailable = (): boolean => {
  if (typeof window === 'undefined') return false;
  return typeof (window as any).freighter !== 'undefined';
};

/**
 * Check if wallet is connected (Freighter)
 */
export const isWalletConnected = async (): Promise<boolean> => {
  if (typeof window === 'undefined') return false;
  try {
    const connectionResult = await freighterIsConnected();
    if (!connectionResult.isConnected) return false;
    const allowedResult = await isAllowed();
    return allowedResult.isAllowed;
  } catch {
    return false;
  }
};

/**
 * Connect wallet - requests access from Freighter
 */
export const connectWallet = async (): Promise<string> => {
  const connectionResult = await freighterIsConnected();
  if (!connectionResult.isConnected) {
    throw new Error('Freighter wallet not found. Please install the Freighter extension.');
  }

  const accessResult = await requestAccess();
  if (accessResult.error) {
    throw new Error(accessResult.error);
  }

  const addrResult = await getAddress();
  if (addrResult.error || !addrResult.address) {
    throw new Error(addrResult.error || 'Could not retrieve address from Freighter.');
  }

  return addrResult.address;
};

/**
 * Get current wallet public key
 */
export const getWalletAddress = async (): Promise<string | null> => {
  try {
    const connected = await isWalletConnected();
    if (!connected) return null;
    const result = await getAddress();
    return result.address || null;
  } catch {
    return null;
  }
};

/**
 * Known Stellar network passphrases for mapping to labels
 */
const TESTNET_PASSPHRASE = 'Test SDF Network ; September 2015';
const PUBLIC_PASSPHRASE = 'Public Global Stellar Network ; September 2015';

/**
 * Get human-friendly label for the network Freighter is connected to.
 * Returns "testnet", "public" or "unknown" or null on failure.
 */
export const getFreighterNetworkLabel = async (): Promise<string | null> => {
  try {
    const result = await getNetworkDetails();
    if (result.error || !result.networkPassphrase) return null;
    if (result.networkPassphrase === TESTNET_PASSPHRASE) return 'testnet';
    if (result.networkPassphrase === PUBLIC_PASSPHRASE) return 'public';
    return 'unknown';
  } catch {
    return null;
  }
};

/**
 * Sign a transaction XDR string using Freighter
 * If Freighter network doesn't match the app network, refuse to sign.
 */
export const signTx = async (txXdr: string): Promise<string> => {
  // Ensure network matches before attempting to sign
  const isCorrect = await verifyNetwork();
  if (!isCorrect) {
    const freighterLabel = (await getFreighterNetworkLabel()) || 'unknown';
    // Use CURRENT_NETWORK (app expected network) for message
    throw new Error(
      `Network mismatch: Freighter is on "${freighterLabel}" but app requires "${CURRENT_NETWORK}". Signing is blocked.`,
    );
  }

  const networkPassphrase = getNetworkPassphrase();
  const result = await signTransaction(txXdr, { networkPassphrase });
  if (result.error) {
    throw new Error(result.error);
  }
  return result.signedTxXdr;
};

/**
 * Get current wallet data
 */
export const getWalletData = async (): Promise<WalletData> => {
  const address = await getWalletAddress();
  const connected = await isWalletConnected();
  return {
    address: address || '',
    isConnected: connected,
    network: CURRENT_NETWORK,
  };
};

/**
 * Verify connected network matches expected network
 */
export const verifyNetwork = async (): Promise<boolean> => {
  try {
    const result = await getNetworkDetails();
    if (result.error) return false;
    const passphrase = getNetworkPassphrase();
    return result.networkPassphrase === passphrase;
  } catch {
    return false;
  }
};

/**
 * Format Stellar address for display (truncated)
 */
export const formatAddress = (address: string, chars = 4): string => {
  if (!address) return '';
  return `${address.slice(0, chars + 1)}...${address.slice(-chars)}`;
};
