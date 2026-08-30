import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@stellar/freighter-api', () => ({
  requestAccess: vi.fn(),
  isAllowed: vi.fn(),
  getAddress: vi.fn(),
  signTransaction: vi.fn(),
  getNetworkDetails: vi.fn(),
  isConnected: vi.fn(),
}));

vi.mock('./stellar-config', () => ({
  CURRENT_NETWORK: 'testnet',
  getNetworkPassphrase: vi.fn().mockReturnValue('Test SDF Network ; September 2015'),
}));

import {
  isWalletConnected,
  connectWallet,
  getWalletAddress,
  formatAddress,
  getFreighterNetworkLabel,
  verifyNetwork,
  signTx,
  getWalletData,
} from './wallet';

import {
  requestAccess,
  isAllowed,
  getAddress,
  signTransaction,
  getNetworkDetails,
  isConnected as freighterIsConnected,
} from '@stellar/freighter-api';

beforeEach(() => {
  vi.clearAllMocks();
});

describe('isWalletConnected', () => {
  it('returns false when freighter reports not connected', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue({ isConnected: false } as any);
    expect(await isWalletConnected()).toBe(false);
  });

  it('returns false when isAllowed returns false', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue({ isConnected: true } as any);
    vi.mocked(isAllowed).mockResolvedValue({ isAllowed: false } as any);
    expect(await isWalletConnected()).toBe(false);
  });

  it('returns true when connected and allowed', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue({ isConnected: true } as any);
    vi.mocked(isAllowed).mockResolvedValue({ isAllowed: true } as any);
    expect(await isWalletConnected()).toBe(true);
  });

  it('returns false on error', async () => {
    vi.mocked(freighterIsConnected).mockRejectedValue(new Error('fail'));
    expect(await isWalletConnected()).toBe(false);
  });
});

describe('connectWallet', () => {
  it('throws if freighter not connected', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue({ isConnected: false } as any);
    await expect(connectWallet()).rejects.toThrow('Freighter wallet not found');
  });

  it('throws on requestAccess error', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue({ isConnected: true } as any);
    vi.mocked(requestAccess).mockResolvedValue({ error: 'denied' } as any);
    await expect(connectWallet()).rejects.toThrow('denied');
  });

  it('returns address on success', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue({ isConnected: true } as any);
    vi.mocked(requestAccess).mockResolvedValue({} as any);
    vi.mocked(getAddress).mockResolvedValue({ address: 'GABC123', error: undefined } as any);
    expect(await connectWallet()).toBe('GABC123');
  });

  it('throws if getAddress fails', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue({ isConnected: true } as any);
    vi.mocked(requestAccess).mockResolvedValue({} as any);
    vi.mocked(getAddress).mockResolvedValue({ address: undefined, error: 'no addr' } as any);
    await expect(connectWallet()).rejects.toThrow('no addr');
  });
});

describe('getWalletAddress', () => {
  it('returns null when not connected', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue({ isConnected: false } as any);
    expect(await getWalletAddress()).toBeNull();
  });

  it('returns address when connected', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue({ isConnected: true } as any);
    vi.mocked(isAllowed).mockResolvedValue({ isAllowed: true } as any);
    vi.mocked(getAddress).mockResolvedValue({ address: 'GADDR' } as any);
    expect(await getWalletAddress()).toBe('GADDR');
  });
});

describe('getFreighterNetworkLabel', () => {
  it('returns testnet for testnet passphrase', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue({
      networkPassphrase: 'Test SDF Network ; September 2015',
    } as any);
    expect(await getFreighterNetworkLabel()).toBe('testnet');
  });

  it('returns public for public passphrase', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue({
      networkPassphrase: 'Public Global Stellar Network ; September 2015',
    } as any);
    expect(await getFreighterNetworkLabel()).toBe('public');
  });

  it('returns unknown for other passphrase', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue({
      networkPassphrase: 'Something else',
    } as any);
    expect(await getFreighterNetworkLabel()).toBe('unknown');
  });

  it('returns null on error', async () => {
    vi.mocked(getNetworkDetails).mockRejectedValue(new Error('fail'));
    expect(await getFreighterNetworkLabel()).toBeNull();
  });
});

describe('verifyNetwork', () => {
  it('returns true when network matches', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue({
      networkPassphrase: 'Test SDF Network ; September 2015',
    } as any);
    expect(await verifyNetwork()).toBe(true);
  });

  it('returns false when network mismatches', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue({
      networkPassphrase: 'Wrong Network',
    } as any);
    expect(await verifyNetwork()).toBe(false);
  });

  it('returns false on error', async () => {
    vi.mocked(getNetworkDetails).mockRejectedValue(new Error('fail'));
    expect(await verifyNetwork()).toBe(false);
  });
});

describe('signTx', () => {
  it('throws on network mismatch', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue({
      networkPassphrase: 'Wrong Network',
    } as any);
    await expect(signTx('xdr')).rejects.toThrow('Network mismatch');
  });

  it('signs and returns XDR on success', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue({
      networkPassphrase: 'Test SDF Network ; September 2015',
    } as any);
    vi.mocked(signTransaction).mockResolvedValue({ signedTxXdr: 'signed_xdr', error: undefined } as any);
    expect(await signTx('xdr')).toBe('signed_xdr');
  });

  it('throws on sign error', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue({
      networkPassphrase: 'Test SDF Network ; September 2015',
    } as any);
    vi.mocked(signTransaction).mockResolvedValue({ signedTxXdr: undefined, error: 'sign failed' } as any);
    await expect(signTx('xdr')).rejects.toThrow('sign failed');
  });
});

describe('getWalletData', () => {
  it('returns connected data', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue({ isConnected: true } as any);
    vi.mocked(isAllowed).mockResolvedValue({ isAllowed: true } as any);
    vi.mocked(getAddress).mockResolvedValue({ address: 'GADDR' } as any);
    const data = await getWalletData();
    expect(data.address).toBe('GADDR');
    expect(data.isConnected).toBe(true);
    expect(data.network).toBe('testnet');
  });
});

describe('formatAddress', () => {
  it('truncates address', () => {
    // formatAddress slices (0, chars+1) from start and (-chars) from end
    expect(formatAddress('GBDFPGJKLMNOPQRSTUVWXYZ1234567890ABCDE')).toBe('GBDFP...BCDE');
  });

  it('returns empty string for empty input', () => {
    expect(formatAddress('')).toBe('');
  });

  it('respects custom char count', () => {
    expect(formatAddress('GBDFPGJKLMNOPQRSTUVWXYZ1234567890ABCDE', 6)).toBe('GBDFPGJ...0ABCDE');
  });
});
