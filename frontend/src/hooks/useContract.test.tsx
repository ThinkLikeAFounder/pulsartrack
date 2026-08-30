import { renderHook, act, waitFor } from '@testing-library/react';
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
    u32ToScVal: vi.fn((v) => ({ _type: 'u32', value: v })),
    stringToScVal: vi.fn((v) => ({ _type: 'string', value: v })),
    i128ToScVal: vi.fn((v) => ({ _type: 'i128', value: v })),
    addressToScVal: vi.fn((v) => ({ _type: 'address', value: v })),
    boolToScVal: vi.fn((v) => ({ _type: 'bool', value: v })),
}));

// Mock stellar-config so CONTRACT_IDS are non-empty (enables React Query)
vi.mock('@/lib/stellar-config', () => ({
    CONTRACT_IDS: {
        CAMPAIGN_ORCHESTRATOR: 'CAMPAIGN_CONTRACT_ID',
        AUCTION_ENGINE: 'AUCTION_CONTRACT_ID',
        AD_REGISTRY: 'AD_REGISTRY_ID',
        ESCROW_VAULT: 'ESCROW_VAULT_ID',
        FRAUD_PREVENTION: 'FRAUD_PREVENTION_ID',
        PAYMENT_PROCESSOR: 'PAYMENT_PROCESSOR_ID',
        GOVERNANCE_TOKEN: 'GOVERNANCE_TOKEN_ID',
        GOVERNANCE_DAO: 'GOVERNANCE_DAO_ID',
        PUBLISHER_VERIFICATION: 'PUBLISHER_VERIFICATION_ID',
        PUBLISHER_REPUTATION: 'PUBLISHER_REPUTATION_ID',
        ANALYTICS_AGGREGATOR: 'ANALYTICS_AGGREGATOR_ID',
        SUBSCRIPTION_MANAGER: 'SUBSCRIPTION_MANAGER_ID',
        PRIVACY_LAYER: 'PRIVACY_LAYER_ID',
        TARGETING_ENGINE: 'TARGETING_ENGINE_ID',
        IDENTITY_REGISTRY: 'IDENTITY_REGISTRY_ID',
        DISPUTE_RESOLUTION: 'DISPUTE_RESOLUTION_ID',
        REVENUE_SETTLEMENT: 'REVENUE_SETTLEMENT_ID',
        REWARDS_DISTRIBUTOR: 'REWARDS_DISTRIBUTOR_ID',
    },
    STROOPS_PER_XLM: 10_000_000,
    stroopsToXlm: (stroops: bigint | number) => Number(stroops) / 10_000_000,
    xlmToStroops: (xlm: number) => BigInt(Math.floor(xlm * 10_000_000)),
}));

const createWrapper = () => {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                retry: false,
            },
        },
    });
    function Wrapper({ children }: { children: React.ReactNode }) {
        return (
            <QueryClientProvider client={queryClient}>
                {children}
            </QueryClientProvider>
        );
    }
    return Wrapper;
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

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual(mockCampaign);
            expect(callReadOnly).toHaveBeenCalled();
        });

        it('should handle fetch error', async () => {
            vi.mocked(callReadOnly).mockRejectedValue(new Error('Contract error'));

            const { result } = renderHook(() => useCampaign(1), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isError).toBe(true);
            });

            expect(result.current.error).toBeDefined();
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
                    campaignType: 1,
                    budgetXlm: 100,
                    costPerViewXlm: 0.001,
                    durationDays: 30,
                    targetViews: 100000,
                    dailyViewLimit: 5000,
                    refundable: true,
                });
            });

            expect(callContract).toHaveBeenCalled();
        });
    });
});
