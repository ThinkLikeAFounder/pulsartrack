import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/lib/wallet', () => ({
  isWalletConnected: vi.fn(),
  getWalletAddress: vi.fn(),
  verifyNetwork: vi.fn(),
  getFreighterNetworkLabel: vi.fn(),
}));

import {
  isWalletConnected,
  getWalletAddress,
  verifyNetwork,
  getFreighterNetworkLabel,
} from '@/lib/wallet';
import { useWalletStore } from './wallet-store';

const mockIsWalletConnected = vi.mocked(isWalletConnected);
const mockGetWalletAddress = vi.mocked(getWalletAddress);
const mockVerifyNetwork = vi.mocked(verifyNetwork);
const mockGetFreighterNetworkLabel = vi.mocked(getFreighterNetworkLabel);

const INITIAL = {
  address: null,
  isConnected: false,
  network: 'testnet',
  freighterNetwork: null,
  networkMismatch: false,
} as const;

function reset() {
  useWalletStore.setState({ ...INITIAL });
}

describe('wallet-store', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    reset();
  });

  describe('setters', () => {
    it('updates individual fields', () => {
      const s = useWalletStore.getState();
      s.setAddress('GABC');
      s.setConnected(true);
      s.setNetwork('mainnet');
      s.setFreighterNetwork('Mainnet');
      s.setNetworkMismatch(true);

      const state = useWalletStore.getState();
      expect(state.address).toBe('GABC');
      expect(state.isConnected).toBe(true);
      expect(state.network).toBe('mainnet');
      expect(state.freighterNetwork).toBe('Mainnet');
      expect(state.networkMismatch).toBe(true);
    });
  });

  describe('disconnect', () => {
    it('clears connection state', () => {
      useWalletStore.setState({
        address: 'GABC',
        isConnected: true,
        networkMismatch: true,
        freighterNetwork: 'Testnet',
      });

      useWalletStore.getState().disconnect();

      const state = useWalletStore.getState();
      expect(state.address).toBeNull();
      expect(state.isConnected).toBe(false);
      expect(state.networkMismatch).toBe(false);
      expect(state.freighterNetwork).toBeNull();
    });
  });

  describe('autoReconnect', () => {
    it('restores address and connection when the wallet is connected on the right network', async () => {
      mockIsWalletConnected.mockResolvedValue(true);
      mockGetWalletAddress.mockResolvedValue('GABC');
      mockVerifyNetwork.mockResolvedValue(true);
      mockGetFreighterNetworkLabel.mockResolvedValue('Testnet');

      await useWalletStore.getState().autoReconnect();

      const state = useWalletStore.getState();
      expect(state.address).toBe('GABC');
      expect(state.isConnected).toBe(true);
      expect(state.freighterNetwork).toBe('Testnet');
      expect(state.networkMismatch).toBe(false);
    });

    it('flags a network mismatch and stays disconnected when on the wrong network', async () => {
      mockIsWalletConnected.mockResolvedValue(true);
      mockGetWalletAddress.mockResolvedValue('GABC');
      mockVerifyNetwork.mockResolvedValue(false);
      mockGetFreighterNetworkLabel.mockResolvedValue('Mainnet');

      await useWalletStore.getState().autoReconnect();

      const state = useWalletStore.getState();
      expect(state.address).toBe('GABC');
      expect(state.isConnected).toBe(false);
      expect(state.networkMismatch).toBe(true);
    });

    it('clears state when the wallet is not connected', async () => {
      useWalletStore.setState({ address: 'GABC', isConnected: true });
      mockIsWalletConnected.mockResolvedValue(false);

      await useWalletStore.getState().autoReconnect();

      const state = useWalletStore.getState();
      expect(state.address).toBeNull();
      expect(state.isConnected).toBe(false);
      expect(mockGetWalletAddress).not.toHaveBeenCalled();
    });

    it('logs and resets state instead of silently swallowing errors (issue #790)', async () => {
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
      useWalletStore.setState({ address: 'GABC', isConnected: true });
      mockIsWalletConnected.mockRejectedValue(new Error('network down'));

      await expect(useWalletStore.getState().autoReconnect()).resolves.toBeUndefined();

      expect(consoleError).toHaveBeenCalledWith(
        'Wallet auto-reconnect failed:',
        expect.any(Error),
      );
      const state = useWalletStore.getState();
      expect(state.address).toBeNull();
      expect(state.isConnected).toBe(false);

      consoleError.mockRestore();
    });
  });
});
