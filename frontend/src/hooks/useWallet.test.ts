import { renderHook, act } from '@testing-library/react';
import { useWallet } from './useWallet';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import {
    connectWallet,
    isWalletConnected,
    getWalletAddress,
    verifyNetwork,
    getFreighterNetworkLabel,
    getWalletData,
} from '@/lib/wallet';
import { useWalletStore } from '@/store/wallet-store';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';

// Mock the wallet lib
vi.mock('@/lib/wallet', () => ({
    connectWallet: vi.fn(),
    isWalletConnected: vi.fn(),
    getWalletAddress: vi.fn(),
    verifyNetwork: vi.fn(),
    getFreighterNetworkLabel: vi.fn(),
    getWalletData: vi.fn(),
}));

vi.mock('@/lib/error-handler', () => ({
    parseStellarError: vi.fn((err: Error) => err.message),
}));

const createWrapper = () => {
    const queryClient = new QueryClient({
        defaultOptions: { queries: { retry: false } },
    });
    function Wrapper({ children }: { children: React.ReactNode }) {
        return React.createElement(QueryClientProvider, { client: queryClient }, children);
    }
    return Wrapper;
};

describe('useWallet', () => {
    beforeEach(() => {
        // Reset store state before each test
        const store = useWalletStore.getState();
        act(() => {
            store.disconnect();
        });
        vi.clearAllMocks();
        // Apply default mock implementations
        vi.mocked(isWalletConnected).mockResolvedValue(false);
        vi.mocked(verifyNetwork).mockResolvedValue(true);
        vi.mocked(getFreighterNetworkLabel).mockResolvedValue('TESTNET');
        vi.mocked(getWalletData).mockResolvedValue({ network: 'testnet', address: '', isConnected: false });
    });

    it('should connect successfully', async () => {
        const mockAddress = 'GABC...123';
        vi.mocked(connectWallet).mockResolvedValue(mockAddress);

        const { result } = renderHook(() => useWallet(), { wrapper: createWrapper() });

        await act(async () => {
            const resp = await result.current.connect();
            expect(resp.success).toBe(true);
            expect(resp.address).toBe(mockAddress);
        });

        expect(result.current.address).toBe(mockAddress);
        expect(result.current.isConnected).toBe(true);
    });

    it('should handle connection error', async () => {
        vi.mocked(connectWallet).mockRejectedValue(new Error('User rejected'));

        const { result } = renderHook(() => useWallet(), { wrapper: createWrapper() });

        await act(async () => {
            const resp = await result.current.connect();
            expect(resp.success).toBe(false);
            expect(resp.error).toBe('User rejected');
        });

        expect(result.current.isConnected).toBe(false);
    });

    it('should disconnect successfully', () => {
        // Set initial state
        act(() => {
            useWalletStore.getState().setAddress('GABC...123');
            useWalletStore.getState().setConnected(true);
        });

        const { result } = renderHook(() => useWallet(), { wrapper: createWrapper() });
        expect(result.current.isConnected).toBe(true);

        act(() => {
            result.current.disconnect();
        });

        expect(result.current.isConnected).toBe(false);
        expect(result.current.address).toBe(null);
    });

    it('should check connection and update state if connected', async () => {
        const mockAddress = 'GABC...123';
        vi.mocked(isWalletConnected).mockResolvedValue(true);
        vi.mocked(getWalletAddress).mockResolvedValue(mockAddress);

        const { result } = renderHook(() => useWallet(), { wrapper: createWrapper() });

        await act(async () => {
            await result.current.checkConnection();
        });

        expect(result.current.isConnected).toBe(true);
        expect(result.current.address).toBe(mockAddress);
    });
});
