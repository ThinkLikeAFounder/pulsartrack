import { renderHook, act } from '@testing-library/react';
import { useCampaign, useCreateCampaign } from './useContract';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { callReadOnly, callContract } from '@/lib/soroban-client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { useWalletStore } from '@/store/wallet-store';

// Mock soroban-client
vi.mock('@/lib/soroban-client', () => ({
    callReadOnly: vi.fn(),
    callContract: vi.fn(),
    u64ToScVal: vi.fn((v) => ({ _type: 'u64', value: v })),
    stringToScVal: vi.fn((v) => ({ _type: 'string', value: v })),
    i128ToScVal: vi.fn((v) => ({ _type: 'i128', value: v })),
    addressToScVal: vi.fn((v) => ({ _type: 'address', value: v })),
}));

const createWrapper = () => {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                retry: false,
            },
        },
    });
    return ({ children }: { children: React.ReactNode }) => (
        <QueryClientProvider client= { queryClient } >
        { children }
        </QueryClientProvider>
    );
};

describe('useContract hooks', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        useWalletStore.getState().setAddress('GABC...123');
        useWalletStore.getState().setConnected(true);
    });

    describe('useCampaign', () => {
        it('should fetch campaign data successfully', async () => {
            const mockCampaign = { id: 1, title: 'Test Campaign' };
            vi.mocked(callReadOnly).mockResolvedValue(mockCampaign);

            const { result } = renderHook(() => useCampaign(1), {
                wrapper: createWrapper(),
            });

            // Wait for query to finish
            await act(async () => {
                await new Promise((resolve) => setTimeout(resolve, 0));
            });

            expect(result.current.data).toEqual(mockCampaign);
            expect(result.current.isLoading).toBe(false);
            expect(callReadOnly).toHaveBeenCalled();
        });

        it('should handle fetch error', async () => {
            vi.mocked(callReadOnly).mockRejectedValue(new Error('Contract error'));

            const { result } = renderHook(() => useCampaign(1), {
                wrapper: createWrapper(),
            });

            await act(async () => {
                await new Promise((resolve) => setTimeout(resolve, 0));
            });

            expect(result.current.error).toBeDefined();
            expect(result.current.isError).toBe(true);
        });
    });

    describe('useCreateCampaign', () => {
        it('should call contract to create campaign', async () => {
            vi.mocked(callContract).mockResolvedValue({ success: true, result: 123 });

            const { result } = renderHook(() => useCreateCampaign(), {
                wrapper: createWrapper(),
            });

            await act(async () => {
                await result.current.createCampaign({
                    title: 'New Campaign',
                    budgetXlm: 100,
                    dailyBudgetXlm: 10,
                    durationDays: 30,
                    contentId: 'ipfs://...',
                    campaignType: 1,
                });
            });

            expect(callContract).toHaveBeenCalled();
        });
    });
});
