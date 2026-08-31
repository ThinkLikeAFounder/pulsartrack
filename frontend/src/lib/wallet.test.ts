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

type ConnectedResult = Awaited<ReturnType<typeof freighterIsConnected>>;
type AllowedResult = Awaited<ReturnType<typeof isAllowed>>;
type AccessResult = Awaited<ReturnType<typeof requestAccess>>;
type AddressResult = Awaited<ReturnType<typeof getAddress>>;
type NetworkDetailsResult = Awaited<ReturnType<typeof getNetworkDetails>>;
type SignResult = Awaited<ReturnType<typeof signTransaction>>;

const connectedResult = (isConnected: boolean): ConnectedResult =>
  ({ isConnected }) as ConnectedResult;
const allowedResult = (isAllowedValue: boolean): AllowedResult =>
  ({ isAllowed: isAllowedValue }) as AllowedResult;
const accessResult = (error?: string): AccessResult => ({ error }) as AccessResult;
const addressResult = (address?: string, error?: string): AddressResult =>
  ({ address, error }) as AddressResult;
const networkResult = (networkPassphrase: string): NetworkDetailsResult =>
  ({ networkPassphrase }) as NetworkDetailsResult;
const signResult = (signedTxXdr?: string, error?: string): SignResult =>
  ({ signedTxXdr, error }) as SignResult;

beforeEach(() => {
  vi.clearAllMocks();
});

describe('isWalletConnected', () => {
  it('returns false when freighter reports not connected', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue(connectedResult(false));
    expect(await isWalletConnected()).toBe(false);
  });

  it('returns false when isAllowed returns false', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue(connectedResult(true));
    vi.mocked(isAllowed).mockResolvedValue(allowedResult(false));
    expect(await isWalletConnected()).toBe(false);
  });

  it('returns true when connected and allowed', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue(connectedResult(true));
    vi.mocked(isAllowed).mockResolvedValue(allowedResult(true));
    expect(await isWalletConnected()).toBe(true);
  });

  it('returns false on error', async () => {
    vi.mocked(freighterIsConnected).mockRejectedValue(new Error('fail'));
    expect(await isWalletConnected()).toBe(false);
  });
});

describe('connectWallet', () => {
  it('throws if freighter not connected', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue(connectedResult(false));
    await expect(connectWallet()).rejects.toThrow('Freighter wallet not found');
  });

  it('throws on requestAccess error', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue(connectedResult(true));
    vi.mocked(requestAccess).mockResolvedValue(accessResult('denied'));
    await expect(connectWallet()).rejects.toThrow('denied');
  });

  it('returns address on success', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue(connectedResult(true));
    vi.mocked(requestAccess).mockResolvedValue(accessResult());
    vi.mocked(getAddress).mockResolvedValue(addressResult('GABC123'));
    expect(await connectWallet()).toBe('GABC123');
  });

  it('throws if getAddress fails', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue(connectedResult(true));
    vi.mocked(requestAccess).mockResolvedValue(accessResult());
    vi.mocked(getAddress).mockResolvedValue(addressResult(undefined, 'no addr'));
    await expect(connectWallet()).rejects.toThrow('no addr');
  });
});

describe('getWalletAddress', () => {
  it('returns null when not connected', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue(connectedResult(false));
    expect(await getWalletAddress()).toBeNull();
  });

  it('returns address when connected', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue(connectedResult(true));
    vi.mocked(isAllowed).mockResolvedValue(allowedResult(true));
    vi.mocked(getAddress).mockResolvedValue(addressResult('GADDR'));
    expect(await getWalletAddress()).toBe('GADDR');
  });
});

describe('getFreighterNetworkLabel', () => {
  it('returns testnet for testnet passphrase', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue(
      networkResult('Test SDF Network ; September 2015'),
    );
    expect(await getFreighterNetworkLabel()).toBe('testnet');
  });

  it('returns public for public passphrase', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue(
      networkResult('Public Global Stellar Network ; September 2015'),
    );
    expect(await getFreighterNetworkLabel()).toBe('public');
  });

  it('returns unknown for other passphrase', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue(networkResult('Something else'));
    expect(await getFreighterNetworkLabel()).toBe('unknown');
  });

  it('returns null on error', async () => {
    vi.mocked(getNetworkDetails).mockRejectedValue(new Error('fail'));
    expect(await getFreighterNetworkLabel()).toBeNull();
  });
});

describe('verifyNetwork', () => {
  it('returns true when network matches', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue(
      networkResult('Test SDF Network ; September 2015'),
    );
    expect(await verifyNetwork()).toBe(true);
  });

  it('returns false when network mismatches', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue(networkResult('Wrong Network'));
    expect(await verifyNetwork()).toBe(false);
  });

  it('returns false on error', async () => {
    vi.mocked(getNetworkDetails).mockRejectedValue(new Error('fail'));
    expect(await verifyNetwork()).toBe(false);
  });
});

describe('signTx', () => {
  it('throws on network mismatch', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue(networkResult('Wrong Network'));
    await expect(signTx('xdr')).rejects.toThrow('Network mismatch');
  });

  it('signs and returns XDR on success', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue(
      networkResult('Test SDF Network ; September 2015'),
    );
    vi.mocked(signTransaction).mockResolvedValue(signResult('signed_xdr'));
    expect(await signTx('xdr')).toBe('signed_xdr');
  });

  it('throws on sign error', async () => {
    vi.mocked(getNetworkDetails).mockResolvedValue(
      networkResult('Test SDF Network ; September 2015'),
    );
    vi.mocked(signTransaction).mockResolvedValue(signResult(undefined, 'sign failed'));
    await expect(signTx('xdr')).rejects.toThrow('sign failed');
  });
});

describe('getWalletData', () => {
  it('returns connected data', async () => {
    vi.mocked(freighterIsConnected).mockResolvedValue(connectedResult(true));
    vi.mocked(isAllowed).mockResolvedValue(allowedResult(true));
    vi.mocked(getAddress).mockResolvedValue(addressResult('GADDR'));
    const data = await getWalletData();
    expect(data.address).toBe('GADDR');
    expect(data.isConnected).toBe(true);
    expect(data.network).toBe('testnet');
  });
});

describe('formatAddress', () => {
  it('truncates address', () => {
    expect(formatAddress('GBDFPGJKLMNOPQRSTUVWXYZ1234567890ABCDE')).toBe('GBDFP...BCDE');
  });

  it('returns empty string for empty input', () => {
    expect(formatAddress('')).toBe('');
  });

  it('respects custom char count', () => {
    expect(formatAddress('GBDFPGJKLMNOPQRSTUVWXYZ1234567890ABCDE', 6)).toBe('GBDFPGJ...0ABCDE');
  });
});