import { renderHook, act } from '@testing-library/react';
import { useWallet } from './useWallet';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { connectWallet, isWalletConnected, getWalletAddress } from '@/lib/wallet';
import { useWalletStore } from '@/store/wallet-store';

// Mock the wallet lib
vi.mock('@/lib/wallet', () => ({
    connectWallet: vi.fn(),
    isWalletConnected: vi.fn(),
    getWalletAddress: vi.fn(),
    parseStellarError: vi.fn((err: any) => err.message),
}));

describe('useWallet', () => {
    beforeEach(() => {
        // Reset store state before each test
        const store = useWalletStore.getState();
        act(() => {
            store.disconnect();
        });
        vi.clearAllMocks();
    });

    it('should connect successfully', async () => {
        const mockAddress = 'GABC...123';
        vi.mocked(connectWallet).mockResolvedValue(mockAddress);

        const { result } = renderHook(() => useWallet());

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

        const { result } = renderHook(() => useWallet());

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

        const { result } = renderHook(() => useWallet());
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

        const { result } = renderHook(() => useWallet());

        await act(async () => {
            await result.current.checkConnection();
        });

        expect(result.current.isConnected).toBe(true);
        expect(result.current.address).toBe(mockAddress);
    });
});
