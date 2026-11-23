'use client';

import {
  requestAccess,
  isAllowed,
  getAddress,
  signTransaction,
  getNetworkDetails,
  isConnected as freighterIsConnected,
} from '@stellar/freighter-api';
import { CURRENT_NETWORK, getNetworkPassphrase, isDemoMode } from './stellar-config';

export interface WalletData {
  address: string;
  isConnected: boolean;
  network: string;
}

/* ============================================================================
   Demo wallet — used whenever isDemoMode() is on (no deployed contracts yet)
   or Freighter isn't installed, so the UI's connect/sign flow always has a
   complete, interactive demo path. Never a real keypair with real funds.
   ========================================================================== */

const DEMO_ADDRESS_KEY = 'pulsar-demo-wallet-address';
const BASE32_ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';

function generateDemoAddress(): string {
  const bytes = new Uint8Array(55);
  crypto.getRandomValues(bytes);
  let body = '';
  for (const b of bytes) body += BASE32_ALPHABET[b % BASE32_ALPHABET.length];
  return `G${body}`.slice(0, 56);
}

function getOrCreateDemoAddress(): string {
  if (typeof window === 'undefined') return generateDemoAddress();
  try {
    const existing = window.localStorage.getItem(DEMO_ADDRESS_KEY);
    if (existing) return existing;
    const fresh = generateDemoAddress();
    window.localStorage.setItem(DEMO_ADDRESS_KEY, fresh);
    return fresh;
  } catch {
    return generateDemoAddress();
  }
}

function clearDemoAddress(): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.removeItem(DEMO_ADDRESS_KEY);
  } catch {
    // ignore
  }
}

function hasDemoAddress(): boolean {
  if (typeof window === 'undefined') return false;
  try {
    return window.localStorage.getItem(DEMO_ADDRESS_KEY) !== null;
  } catch {
    return false;
  }
}

/** True once "Connect Wallet" has been used in demo mode this browser. */
export const isDemoWalletConnected = (): boolean => isDemoMode() && hasDemoAddress();

export const disconnectDemoWallet = (): void => clearDemoAddress();

/**
 * Check if Freighter wallet extension is available
 */
export const isFreighterAvailable = (): boolean => {
  if (typeof window === 'undefined') return false;
  return typeof (window as Window & { freighter?: unknown }).freighter !== 'undefined';
};

/**
 * Check if wallet is connected (Freighter)
 */
export const isWalletConnected = async (): Promise<boolean> => {
  if (typeof window === 'undefined') return false;
  if (isDemoMode()) return hasDemoAddress();
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
 * Connect wallet - requests access from Freighter, or spins up a persisted
 * demo address when running in demo mode (no deployed contracts yet).
 */
export const connectWallet = async (): Promise<string> => {
  if (isDemoMode()) {
    return getOrCreateDemoAddress();
  }

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
  if (isDemoMode()) return hasDemoAddress() ? getOrCreateDemoAddress() : null;
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
  if (isDemoMode()) return CURRENT_NETWORK;
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
  // Demo mode never reaches here (callContract short-circuits into
  // demoContractCall before any real transaction is built), but keep this
  // safe just in case something calls signTx directly.
  if (isDemoMode()) return txXdr;

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
  if (isDemoMode()) return true;
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
