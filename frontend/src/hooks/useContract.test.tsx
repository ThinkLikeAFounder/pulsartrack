import { renderHook, act, waitFor } from '@testing-library/react';
import {
    useContractCall,
    useContractRead,
    useCampaign,
    usePublisherReputation,
    useAdvertiserStats,
    useCampaignCount,
    useAdvertiserCampaigns,
    useSubscription,
    usePrivacyConsent,
    useAuction,
    useCreateCampaign,
    useCampaign,
    useCreateCampaign,
    useAdvertiserStats,
    useCampaignCount,
    useAdvertiserCampaigns,
    usePublisherReputation,
    useSubscription,
    usePrivacyConsent,
    useAuction,
    usePlaceBid,
    useSetConsent,
    useGovernanceBalance,
    useProposalCount,
    useGovernanceProposal,
    useGovernanceProposals,
    useCastVote,
    useCreateProposal,
    usePublisherData,
    usePublisherKyc,
    usePublisherEarnings,
    useAuctionCount,
    usePublisherAuctions,
    useOpenAuctions,
    useCreateAuction,
    useSubscribe,
    useWallet,
} from './useContract';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { callReadOnly, callContract } from '@/lib/soroban-client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { useWalletStore } from '@/store/wallet-store';

// Mock soroban-client
vi.mock('@/lib/soroban-client', () => ({
    callReadOnly: vi.fn(),
    callContract: vi.fn(),
    u64ToScVal: vi.fn((v) => ({ __type: 'u64', value: v })),
    u32ToScVal: vi.fn((v) => ({ __type: 'u32', value: v })),
    stringToScVal: vi.fn((v) => ({ __type: 'string', value: v })),
    i128ToScVal: vi.fn((v) => ({ __type: 'i128', value: v })),
    addressToScVal: vi.fn((v) => ({ __type: 'address', value: v })),
    boolToScVal: vi.fn((v) => ({ __type: 'bool', value: v })),
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

    describe('useContractCall', () => {
        it('should execute contract call and invalidate queries on success', async () => {
            vi.mocked(callContract).mockResolvedValue({ success: true, result: 42 });

            const { result } = renderHook(() => useContractCall(), {
                wrapper: createWrapper(),
            });

            await act(async () => {
                await result.current.mutateAsync({
                    contractId: 'CAMPAIGN_CONTRACT_ID',
                    method: 'pause_campaign',
                    source: 'GABC...123',
                    args: [],
                });
            });

            expect(callContract).toHaveBeenCalledWith({
                contractId: 'CAMPAIGN_CONTRACT_ID',
                method: 'pause_campaign',
                source: 'GABC...123',
                args: [],
            });
        });

        it('should propagate error when contract call fails', async () => {
            vi.mocked(callContract).mockRejectedValue(new Error('Tx failed'));

            const { result } = renderHook(() => useContractCall(), {
                wrapper: createWrapper(),
            });

            await expect(
                act(async () => {
                    await result.current.mutateAsync({
                        contractId: 'CAMPAIGN_CONTRACT_ID',
                        method: 'pause_campaign',
                        source: 'GABC...123',
                    });
                }),
            ).rejects.toThrow('Tx failed');
        });
    });

    describe('useContractRead', () => {
        it('should read contract state when enabled', async () => {
            vi.mocked(callReadOnly).mockResolvedValue({ ok: true });

            const { result } = renderHook(
                () =>
                    useContractRead({
                        contractId: 'CAMPAIGN_CONTRACT_ID',
                        method: 'get_config',
                        args: [{ __type: 'u32', value: 1 }],
                    }),
                { wrapper: createWrapper() },
            );

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual({ ok: true });
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'CAMPAIGN_CONTRACT_ID',
                method: 'get_config',
                args: [{ __type: 'u32', value: 1 }],
            });
        });
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

    describe('usePublisherReputation', () => {
        it('should fetch publisher reputation', async () => {
            const mockRep = { score: 850 };
            vi.mocked(callReadOnly).mockResolvedValue(mockRep);

            const { result } = renderHook(() => usePublisherReputation('GABC...123'), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual(mockRep);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'PUBLISHER_REPUTATION_ID',
                method: 'get_reputation',
                args: [{ __type: 'address', value: 'GABC...123' }],
            });
        });
    });

    describe('useAdvertiserStats', () => {
        it('should fetch advertiser stats', async () => {
            const mockStats = { active_campaigns: 2, total_views: 500, total_spent: 1000n };
            vi.mocked(callReadOnly).mockResolvedValue(mockStats);

            const { result } = renderHook(() => useAdvertiserStats('GABC...123'), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual(mockStats);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'CAMPAIGN_CONTRACT_ID',
                method: 'get_advertiser_stats',
                args: [{ __type: 'address', value: 'GABC...123' }],
            });
        });
    });

    describe('useCampaignCount', () => {
        it('should fetch total campaign count', async () => {
            vi.mocked(callReadOnly).mockResolvedValue(5);

            const { result } = renderHook(() => useCampaignCount(), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toBe(5);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'CAMPAIGN_CONTRACT_ID',
                method: 'get_campaign_count',
                args: [],
            });
        });
    });

    describe('useAdvertiserCampaigns', () => {
        it('should fetch and filter campaigns belonging to the advertiser sorted by descending ID', async () => {
            vi.mocked(callReadOnly)
                .mockResolvedValueOnce({
                    advertiser: 'GABC...123',
                    budget: 100n,
                    target_views: 1000,
                    current_views: 250,
                    status: 'Active',
                })
                .mockResolvedValueOnce({
                    advertiser: 'GOTHER...999',
                    budget: 200n,
                })
                .mockResolvedValueOnce({
                    advertiser: 'GABC...123',
                    budget: 300n,
                    target_views: 2000,
                    current_views: 500,
                    status: { Paused: {} },
                });

            const { result } = renderHook(
                () => useAdvertiserCampaigns('GABC...123', 3),
                { wrapper: createWrapper() },
            );

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual([
                {
                    id: 3,
                    advertiser: 'GABC...123',
                    budget: 300n,
                    target_views: 2000,
                    current_views: 500,
                    status: { Paused: {} },
                },
                {
                    id: 1,
                    advertiser: 'GABC...123',
                    budget: 100n,
                    target_views: 1000,
                    current_views: 250,
                    status: 'Active',
                },
            ]);
        });
    });

    describe('useSubscription', () => {
        it('should fetch subscriber subscription status', async () => {
            const mockSub = { tier: 'Growth', is_annual: false };
            vi.mocked(callReadOnly).mockResolvedValue(mockSub);

            const { result } = renderHook(() => useSubscription('GABC...123'), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual(mockSub);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'SUBSCRIPTION_MANAGER_ID',
                method: 'get_subscription',
                args: [{ __type: 'address', value: 'GABC...123' }],
            });
        });
    });

    describe('usePrivacyConsent', () => {
        it('should fetch user privacy consent', async () => {
            const mockConsent = { data_processing: true, targeted_ads: false };
            vi.mocked(callReadOnly).mockResolvedValue(mockConsent);

            const { result } = renderHook(() => usePrivacyConsent('GABC...123'), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual(mockConsent);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'PRIVACY_LAYER_ID',
                method: 'get_consent',
                args: [{ __type: 'address', value: 'GABC...123' }],
            });
        });
    });

    describe('useAuction', () => {
        it('should fetch auction details by ID', async () => {
            const mockAuction = { id: 7, status: 'Open' };
            vi.mocked(callReadOnly).mockResolvedValue(mockAuction);

            const { result } = renderHook(() => useAuction(7), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual(mockAuction);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'AUCTION_CONTRACT_ID',
                method: 'get_auction',
                args: [{ __type: 'u64', value: 7 }],
            });
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

        it('should throw when wallet is not connected', async () => {
            useWalletStore.getState().setAddress(null);

            const { result } = renderHook(() => useCreateCampaign(), {
                wrapper: createWrapper(),
            });

            await expect(
                result.current.createCampaign({
                    campaignType: 1,
                    budgetXlm: 100,
                    costPerViewXlm: 0.001,
                    durationDays: 30,
                    targetViews: 1000,
                    dailyViewLimit: 100,
                    refundable: true,
                }),
            ).rejects.toThrow('Wallet not connected');
        });
    });

    describe('usePlaceBid', () => {
        it('should place a bid on an auction', async () => {
            vi.mocked(callContract).mockResolvedValue({ success: true });

            const { result } = renderHook(() => usePlaceBid(), {
                wrapper: createWrapper(),
            });

            await act(async () => {
                await result.current.placeBid({
                    auctionId: 4,
                    amountStroops: 50_000_000n,
                    campaignId: 2,
                });
            });

            expect(callContract).toHaveBeenCalledWith({
                contractId: 'AUCTION_CONTRACT_ID',
                method: 'place_bid',
                source: 'GABC...123',
                args: [
                    { __type: 'address', value: 'GABC...123' },
                    { __type: 'u64', value: 4 },
                    { __type: 'i128', value: 50_000_000n },
                    { __type: 'u64', value: 2 },
                ],
            });
        });

        it('should throw when wallet is not connected', async () => {
            useWalletStore.getState().setAddress(null);

            const { result } = renderHook(() => usePlaceBid(), {
                wrapper: createWrapper(),
            });

            await expect(
                result.current.placeBid({
                    auctionId: 4,
                    amountStroops: 50_000_000n,
                    campaignId: 2,
                }),
            ).rejects.toThrow('Wallet not connected');
        });
    });

    describe('useSetConsent', () => {
        it('should invoke set_consent with consent flags', async () => {
            vi.mocked(callContract).mockResolvedValue({ success: true });

            const { result } = renderHook(() => useSetConsent(), {
                wrapper: createWrapper(),
            });

            act(() => {
                result.current.setConsent({
                    dataProcessing: true,
                    targetedAds: true,
                    analytics: false,
                    thirdPartySharing: false,
                });
            });

            await waitFor(() => {
                expect(callContract).toHaveBeenCalledWith({
                    contractId: 'PRIVACY_LAYER_ID',
                    method: 'set_consent',
                    source: 'GABC...123',
                    args: [
                        { __type: 'address', value: 'GABC...123' },
                        { __type: 'bool', value: true },
                        { __type: 'bool', value: true },
                        { __type: 'bool', value: false },
                        { __type: 'bool', value: false },
                        { __type: 'u64', value: 0 },
                    ],
                });
            });
        });

        it('should be a no-op when wallet is not connected', () => {
            useWalletStore.getState().setAddress(null);

            const { result } = renderHook(() => useSetConsent(), {
                wrapper: createWrapper(),
            });

            act(() => {
                result.current.setConsent({
                    dataProcessing: true,
                    targetedAds: false,
                    analytics: false,
                    thirdPartySharing: false,
                });
            });

            expect(callContract).not.toHaveBeenCalled();
        });
    });

    describe('useGovernanceBalance', () => {
        it('should fetch governance token balance', async () => {
            vi.mocked(callReadOnly).mockResolvedValue(1_000_000_000n);

            const { result } = renderHook(() => useGovernanceBalance('GABC...123'), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toBe(1_000_000_000n);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'GOVERNANCE_TOKEN_ID',
                method: 'balance',
                args: [{ __type: 'address', value: 'GABC...123' }],
            });
        });
    });

    describe('useProposalCount', () => {
        it('should fetch governance proposal count', async () => {
            vi.mocked(callReadOnly).mockResolvedValue(3);

            const { result } = renderHook(() => useProposalCount(), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toBe(3);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'GOVERNANCE_DAO_ID',
                method: 'get_proposal_count',
                args: [],
            });
        });
    });

    describe('useGovernanceProposal', () => {
        it('should fetch a single governance proposal', async () => {
            const mockProp = { title: 'Increase quorum' };
            vi.mocked(callReadOnly).mockResolvedValue(mockProp);

            const { result } = renderHook(() => useGovernanceProposal(2), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual(mockProp);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'GOVERNANCE_DAO_ID',
                method: 'get_proposal',
                args: [{ __type: 'u64', value: 2 }],
            });
        });
    });

    describe('useGovernanceProposals', () => {
        it('should fetch all proposals sorted in descending ID order', async () => {
            vi.mocked(callReadOnly)
                .mockResolvedValueOnce({ title: 'First Proposal' })
                .mockResolvedValueOnce({ title: 'Second Proposal' });

            const { result } = renderHook(() => useGovernanceProposals(2), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual([
                { id: 2, title: 'Second Proposal' },
                { id: 1, title: 'First Proposal' },
            ]);
        });
    });

    describe('useCastVote', () => {
        it('should submit a governance vote', async () => {
            vi.mocked(callContract).mockResolvedValue({ success: true });

            const { result } = renderHook(() => useCastVote(), {
                wrapper: createWrapper(),
            });

            await act(async () => {
                await result.current.castVote({
                    proposalId: 1,
                    voteType: 'For',
                    votePower: 500n,
                });
            });

            expect(callContract).toHaveBeenCalledWith({
                contractId: 'GOVERNANCE_DAO_ID',
                method: 'cast_vote',
                source: 'GABC...123',
                args: [
                    { __type: 'address', value: 'GABC...123' },
                    { __type: 'u64', value: 1 },
                    { __type: 'string', value: 'For' },
                    { __type: 'i128', value: 500n },
                ],
            });
        });

        it('should throw when wallet is not connected', async () => {
            useWalletStore.getState().setAddress(null);

            const { result } = renderHook(() => useCastVote(), {
                wrapper: createWrapper(),
            });

            await expect(
                result.current.castVote({
                    proposalId: 1,
                    voteType: 'Against',
                    votePower: 100n,
                }),
            ).rejects.toThrow('Wallet not connected');
        });
    });

    describe('useCreateProposal', () => {
        it('should create a governance proposal with voting period converted to ledgers', async () => {
            vi.mocked(callContract).mockResolvedValue({ success: true, result: 1 });

            const { result } = renderHook(() => useCreateProposal(), {
                wrapper: createWrapper(),
            });

            await act(async () => {
                await result.current.createProposal({
                    title: 'Proposal Title',
                    description: 'Proposal Description',
                    votingPeriodDays: 7,
                });
            });

            expect(callContract).toHaveBeenCalledWith({
                contractId: 'GOVERNANCE_DAO_ID',
                method: 'create_proposal',
                source: 'GABC...123',
                args: [
                    { __type: 'address', value: 'GABC...123' },
                    { __type: 'string', value: 'Proposal Title' },
                    { __type: 'string', value: 'Proposal Description' },
                    { __type: 'u64', value: 7 * 17_280 },
                ],
            });
        });

        it('should throw when wallet is not connected', async () => {
            useWalletStore.getState().setAddress(null);

            const { result } = renderHook(() => useCreateProposal(), {
                wrapper: createWrapper(),
            });

            await expect(
                result.current.createProposal({
                    title: 'Proposal Title',
                    description: 'Proposal Description',
                    votingPeriodDays: 7,
                }),
            ).rejects.toThrow('Wallet not connected');
        });
    });

    describe('usePublisherData', () => {
        it('should fetch publisher profile data', async () => {
            const mockPub = { status: 'Active', impressions_served: 1200 };
            vi.mocked(callReadOnly).mockResolvedValue(mockPub);

            const { result } = renderHook(() => usePublisherData('GABC...123'), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual(mockPub);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'PUBLISHER_VERIFICATION_ID',
                method: 'get_publisher',
                args: [{ __type: 'address', value: 'GABC...123' }],
            });
        });
    });

    describe('usePublisherKyc', () => {
        it('should fetch publisher KYC record', async () => {
            const mockKyc = { verified: true };
            vi.mocked(callReadOnly).mockResolvedValue(mockKyc);

            const { result } = renderHook(() => usePublisherKyc('GABC...123'), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual(mockKyc);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'PUBLISHER_VERIFICATION_ID',
                method: 'get_kyc_record',
                args: [{ __type: 'address', value: 'GABC...123' }],
            });
        });
    });

    describe('usePublisherEarnings', () => {
        it('should fetch publisher earnings balance', async () => {
            vi.mocked(callReadOnly).mockResolvedValue(25_000_000n);

            const { result } = renderHook(() => usePublisherEarnings('GABC...123'), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toBe(25_000_000n);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'REVENUE_SETTLEMENT_ID',
                method: 'get_publisher_balance',
                args: [{ __type: 'address', value: 'GABC...123' }],
            });
        });
    });

    describe('useAuctionCount', () => {
        it('should fetch active auction count', async () => {
            vi.mocked(callReadOnly).mockResolvedValue(4);

            const { result } = renderHook(() => useAuctionCount(), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toBe(4);
            expect(callReadOnly).toHaveBeenCalledWith({
                contractId: 'AUCTION_CONTRACT_ID',
                method: 'get_auction_count',
                args: [],
            });
        });
    });

    describe('usePublisherAuctions', () => {
        it('should fetch and filter auctions belonging to the publisher', async () => {
            vi.mocked(callReadOnly)
                .mockResolvedValueOnce({ publisher: 'GABC...123', status: 'Open', current_bid: 100n })
                .mockResolvedValueOnce({ publisher: 'GOTHER...999', status: 'Open', current_bid: 200n });

            const { result } = renderHook(
                () => usePublisherAuctions('GABC...123', 2),
                { wrapper: createWrapper() },
            );

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual([
                { id: 1, publisher: 'GABC...123', status: 'Open', current_bid: 100n },
            ]);
        });
    });

    describe('useOpenAuctions', () => {
        it('should fetch all auctions with auction_id attached in descending order', async () => {
            vi.mocked(callReadOnly)
                .mockResolvedValueOnce({ publisher: 'GPUB1', status: 'Open' })
                .mockResolvedValueOnce({ publisher: 'GPUB2', status: 'Open' });

            const { result } = renderHook(() => useOpenAuctions(2), {
                wrapper: createWrapper(),
            });

            await waitFor(() => {
                expect(result.current.isLoading).toBe(false);
            });

            expect(result.current.data).toEqual([
                { auction_id: 2n, publisher: 'GPUB2', status: 'Open' },
                { auction_id: 1n, publisher: 'GPUB1', status: 'Open' },
            ]);
        });
    });

    describe('useCreateAuction', () => {
        it('should create an auction with stroops and seconds conversions', async () => {
            vi.mocked(callContract).mockResolvedValue({ success: true, result: 1 });

            const { result } = renderHook(() => useCreateAuction(), {
                wrapper: createWrapper(),
            });

            await act(async () => {
                await result.current.createAuction({
                    impressionSlot: 'hero-banner',
                    floorPriceXlm: 1.5,
                    reservePriceXlm: 0.5,
                    durationHours: 12,
                });
            });

            expect(callContract).toHaveBeenCalledWith({
                contractId: 'AUCTION_CONTRACT_ID',
                method: 'create_auction',
                source: 'GABC...123',
                args: [
                    { __type: 'address', value: 'GABC...123' },
                    { __type: 'string', value: 'hero-banner' },
                    { __type: 'i128', value: 15_000_000 },
                    { __type: 'i128', value: 5_000_000 },
                    { __type: 'u64', value: 43_200 },
                ],
            });
        });

        it('should throw when wallet is not connected', async () => {
            useWalletStore.getState().setAddress(null);

            const { result } = renderHook(() => useCreateAuction(), {
                wrapper: createWrapper(),
            });

            await expect(
                result.current.createAuction({
                    impressionSlot: 'hero-banner',
                    floorPriceXlm: 1,
                    reservePriceXlm: 0.5,
                    durationHours: 24,
                }),
            ).rejects.toThrow('Wallet not connected');
        });
    });

    describe('useSubscribe', () => {
        it('should call subscribe on the subscription manager contract', async () => {
            vi.mocked(callContract).mockResolvedValue({ success: true });

            const { result } = renderHook(() => useSubscribe(), {
                wrapper: createWrapper(),
            });

            await act(async () => {
                await result.current.subscribe({
                    planName: 'Growth',
                    amountXlm: 299,
                });
            });

            expect(callContract).toHaveBeenCalledWith({
                contractId: 'SUBSCRIPTION_MANAGER_ID',
                method: 'subscribe',
                source: 'GABC...123',
                args: [
                    { __type: 'address', value: 'GABC...123' },
                    { __type: 'string', value: 'Growth' },
                    { __type: 'i128', value: 2_990_000_000n },
                ],
            });
        });

        it('should throw when wallet is not connected', async () => {
            useWalletStore.getState().setAddress(null);

            const { result } = renderHook(() => useSubscribe(), {
                wrapper: createWrapper(),
            });

            await expect(
                result.current.subscribe({
                    planName: 'Growth',
                    amountXlm: 299,
                }),
            ).rejects.toThrow('Wallet not connected');
        });
    });

    describe('useWallet', () => {
        it('should return current wallet address and connection status', () => {
            const { result } = renderHook(() => useWallet());

            expect(result.current).toEqual({
                address: 'GABC...123',
                isConnected: true,
            });
        });
    });
});

// ---------------------------------------------------------------------------
// Coverage for the remaining hooks (issue #1267). Read hooks are checked for
// the exact contract/method/args they query; mutation hooks for the exact
// argument order they send, since a swapped argument would corrupt the call.
// ---------------------------------------------------------------------------

const ADDR = 'GABC...123';
const OTHER = 'GOTHER...456';
const address = (v: string) => ({ _type: 'address', value: v });
const u64 = (v: number) => ({ _type: 'u64', value: v });
const u32 = (v: number) => ({ _type: 'u32', value: v });
const str = (v: string) => ({ _type: 'string', value: v });
const i128 = (v: bigint | number) => ({ _type: 'i128', value: v });
const bool = (v: boolean) => ({ _type: 'bool', value: v });

type ReadCase = {
    name: string;
    useCase: () => { data: unknown; isSuccess: boolean; fetchStatus: string };
    contractId: string;
    method: string;
    args: unknown[];
    useDisabled?: () => { fetchStatus: string };
};

const readCases: ReadCase[] = [
    {
        name: 'useAdvertiserStats',
        useCase: () => useAdvertiserStats(ADDR),
        contractId: 'CAMPAIGN_CONTRACT_ID',
        method: 'get_advertiser_stats',
        args: [address(ADDR)],
        useDisabled: () => useAdvertiserStats(''),
    },
    {
        name: 'useCampaignCount',
        useCase: () => useCampaignCount(),
        contractId: 'CAMPAIGN_CONTRACT_ID',
        method: 'get_campaign_count',
        args: [],
        useDisabled: () => useCampaignCount(false),
    },
    {
        name: 'usePublisherReputation',
        useCase: () => usePublisherReputation(ADDR),
        contractId: 'PUBLISHER_REPUTATION_ID',
        method: 'get_reputation',
        args: [address(ADDR)],
        useDisabled: () => usePublisherReputation(''),
    },
    {
        name: 'useSubscription',
        useCase: () => useSubscription(ADDR),
        contractId: 'SUBSCRIPTION_MANAGER_ID',
        method: 'get_subscription',
        args: [address(ADDR)],
        useDisabled: () => useSubscription(''),
    },
    {
        name: 'usePrivacyConsent',
        useCase: () => usePrivacyConsent(ADDR),
        contractId: 'PRIVACY_LAYER_ID',
        method: 'get_consent',
        args: [address(ADDR)],
        useDisabled: () => usePrivacyConsent(''),
    },
    {
        name: 'useAuction',
        useCase: () => useAuction(7),
        contractId: 'AUCTION_CONTRACT_ID',
        method: 'get_auction',
        args: [u64(7)],
        useDisabled: () => useAuction(0),
    },
    {
        name: 'useGovernanceBalance',
        useCase: () => useGovernanceBalance(ADDR),
        contractId: 'GOVERNANCE_TOKEN_ID',
        method: 'balance',
        args: [address(ADDR)],
        useDisabled: () => useGovernanceBalance(''),
    },
    {
        name: 'useProposalCount',
        useCase: () => useProposalCount(),
        contractId: 'GOVERNANCE_DAO_ID',
        method: 'get_proposal_count',
        args: [],
        useDisabled: () => useProposalCount(false),
    },
    {
        name: 'useGovernanceProposal',
        useCase: () => useGovernanceProposal(3),
        contractId: 'GOVERNANCE_DAO_ID',
        method: 'get_proposal',
        args: [u64(3)],
        useDisabled: () => useGovernanceProposal(0),
    },
    {
        name: 'usePublisherData',
        useCase: () => usePublisherData(ADDR),
        contractId: 'PUBLISHER_VERIFICATION_ID',
        method: 'get_publisher',
        args: [address(ADDR)],
        useDisabled: () => usePublisherData(''),
    },
    {
        name: 'usePublisherKyc',
        useCase: () => usePublisherKyc(ADDR),
        contractId: 'PUBLISHER_VERIFICATION_ID',
        method: 'get_kyc_record',
        args: [address(ADDR)],
        useDisabled: () => usePublisherKyc(''),
    },
    {
        name: 'usePublisherEarnings',
        useCase: () => usePublisherEarnings(ADDR),
        contractId: 'REVENUE_SETTLEMENT_ID',
        method: 'get_publisher_balance',
        args: [address(ADDR)],
        useDisabled: () => usePublisherEarnings(''),
    },
    {
        name: 'useAuctionCount',
        useCase: () => useAuctionCount(),
        contractId: 'AUCTION_CONTRACT_ID',
        method: 'get_auction_count',
        args: [],
        useDisabled: () => useAuctionCount(false),
    },
];

describe('useContract read hooks', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it.each(readCases)('$name queries $method with the expected args', async (c) => {
        const payload = { from: c.method };
        vi.mocked(callReadOnly).mockResolvedValue(payload);

        const { result } = renderHook(() => c.useCase(), { wrapper: createWrapper() });

        await waitFor(() => expect(result.current.isSuccess).toBe(true));
        expect(result.current.data).toEqual(payload);
        expect(callReadOnly).toHaveBeenCalledTimes(1);
        expect(callReadOnly).toHaveBeenCalledWith({
            contractId: c.contractId,
            method: c.method,
            args: c.args,
        });
    });

    it.each(readCases.filter((c) => c.useDisabled))(
        '$name does not query when its input is missing or disabled',
        (c) => {
            const { result } = renderHook(() => c.useDisabled!(), { wrapper: createWrapper() });
            expect(result.current.fetchStatus).toBe('idle');
            expect(callReadOnly).not.toHaveBeenCalled();
        },
    );
});

describe('useContract list hooks', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    const byId = (records: Record<number, unknown>) =>
        vi.mocked(callReadOnly).mockImplementation(async (opts) => {
            const arg = opts.args?.[0] as { value: number } | undefined;
            return records[arg?.value ?? 0] ?? null;
        });

    it('useAdvertiserCampaigns keeps only this advertiser\'s campaigns, newest first', async () => {
        byId({
            1: { advertiser: ADDR, budget: 100n, target_views: 10n, current_views: 1n, status: 'Active' },
            2: { advertiser: OTHER, budget: 5n },
            3: { advertiser: ADDR, budget: 300n, status: { Paused: null } },
        });

        const { result } = renderHook(() => useAdvertiserCampaigns(ADDR, 3), {
            wrapper: createWrapper(),
        });

        await waitFor(() => expect(result.current.isSuccess).toBe(true));
        expect(result.current.data?.map((c) => c.id)).toEqual([3, 1]);
        expect(result.current.data?.[1]).toEqual({
            id: 1,
            advertiser: ADDR,
            budget: 100n,
            target_views: 10n,
            current_views: 1n,
            status: 'Active',
        });
        expect(callReadOnly).toHaveBeenCalledWith({
            contractId: 'CAMPAIGN_CONTRACT_ID',
            method: 'get_campaign',
            args: [u64(2)],
        });
    });

    it('useAdvertiserCampaigns stays idle without a campaign count', () => {
        const { result } = renderHook(() => useAdvertiserCampaigns(ADDR, 0), {
            wrapper: createWrapper(),
        });
        expect(result.current.fetchStatus).toBe('idle');
        expect(callReadOnly).not.toHaveBeenCalled();
    });

    it('useGovernanceProposals returns every proposal with its id, newest first', async () => {
        byId({ 1: { title: 'A' }, 2: { title: 'B' } });

        const { result } = renderHook(() => useGovernanceProposals(2), {
            wrapper: createWrapper(),
        });

        await waitFor(() => expect(result.current.isSuccess).toBe(true));
        expect(result.current.data).toEqual([
            { title: 'B', id: 2 },
            { title: 'A', id: 1 },
        ]);
        expect(callReadOnly).toHaveBeenCalledWith({
            contractId: 'GOVERNANCE_DAO_ID',
            method: 'get_proposal',
            args: [u64(1)],
        });
    });

    it('useGovernanceProposals skips a proposal whose read fails', async () => {
        vi.mocked(callReadOnly).mockImplementation(async (opts) => {
            const id = (opts.args?.[0] as { value: number }).value;
            if (id === 1) throw new Error('rpc down');
            return { title: 'B' };
        });

        const { result } = renderHook(() => useGovernanceProposals(2), {
            wrapper: createWrapper(),
        });

        await waitFor(() => expect(result.current.isSuccess).toBe(true));
        expect(result.current.data).toEqual([{ title: 'B', id: 2 }]);
    });

    it('usePublisherAuctions keeps only this publisher\'s auctions', async () => {
        byId({
            1: { publisher: ADDR, status: 'Open' },
            2: { publisher: OTHER, status: 'Open' },
        });

        const { result } = renderHook(() => usePublisherAuctions(ADDR, 2), {
            wrapper: createWrapper(),
        });

        await waitFor(() => expect(result.current.isSuccess).toBe(true));
        expect(result.current.data).toEqual([{ publisher: ADDR, status: 'Open', id: 1 }]);
        expect(callReadOnly).toHaveBeenCalledWith({
            contractId: 'AUCTION_CONTRACT_ID',
            method: 'get_auction',
            args: [u64(2)],
        });
    });

    it('useOpenAuctions returns all auctions tagged with a bigint auction_id', async () => {
        byId({
            1: { publisher: ADDR },
            2: { publisher: OTHER },
        });

        const { result } = renderHook(() => useOpenAuctions(2), { wrapper: createWrapper() });

        await waitFor(() => expect(result.current.isSuccess).toBe(true));
        expect(result.current.data).toEqual([
            { publisher: OTHER, auction_id: 2n },
            { publisher: ADDR, auction_id: 1n },
        ]);
    });
});

describe('useContract mutation hooks', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        useWalletStore.getState().setAddress(ADDR);
        useWalletStore.getState().setConnected(true);
        vi.mocked(callContract).mockResolvedValue({ success: true, txHash: 'abc' });
    });

    it('usePlaceBid sends bidder, auction, amount, campaign in order', async () => {
        const { result } = renderHook(() => usePlaceBid(), { wrapper: createWrapper() });

        await act(async () => {
            await result.current.placeBid({ auctionId: 4, amountStroops: 25_000_000n, campaignId: 9 });
        });

        expect(callContract).toHaveBeenCalledWith({
            contractId: 'AUCTION_CONTRACT_ID',
            method: 'place_bid',
            source: ADDR,
            args: [address(ADDR), u64(4), i128(25_000_000n), u64(9)],
        });
    });

    it('useCastVote sends voter, proposal, vote type, power in order', async () => {
        const { result } = renderHook(() => useCastVote(), { wrapper: createWrapper() });

        await act(async () => {
            await result.current.castVote({ proposalId: 2, voteType: 'Against', votePower: 500n });
        });

        expect(callContract).toHaveBeenCalledWith({
            contractId: 'GOVERNANCE_DAO_ID',
            method: 'cast_vote',
            source: ADDR,
            args: [address(ADDR), u64(2), str('Against'), i128(500n)],
        });
    });

    it('useCreateProposal converts the voting period from days to ledgers', async () => {
        const { result } = renderHook(() => useCreateProposal(), { wrapper: createWrapper() });

        await act(async () => {
            await result.current.createProposal({
                title: 'Raise floor',
                description: 'Details',
                votingPeriodDays: 7,
            });
        });

        expect(callContract).toHaveBeenCalledWith({
            contractId: 'GOVERNANCE_DAO_ID',
            method: 'create_proposal',
            source: ADDR,
            args: [address(ADDR), str('Raise floor'), str('Details'), u64(7 * 17_280)],
        });
    });

    it('useCreateAuction converts prices to stroops and hours to seconds', async () => {
        const { result } = renderHook(() => useCreateAuction(), { wrapper: createWrapper() });

        await act(async () => {
            await result.current.createAuction({
                impressionSlot: 'footer-728x90',
                floorPriceXlm: 1.5,
                reservePriceXlm: 1.25,
                durationHours: 2,
            });
        });

        expect(callContract).toHaveBeenCalledWith({
            contractId: 'AUCTION_CONTRACT_ID',
            method: 'create_auction',
            source: ADDR,
            args: [address(ADDR), str('footer-728x90'), i128(15_000_000), i128(12_500_000), u64(7_200)],
        });
    });

    it('useSubscribe sends subscriber, plan, amount in stroops', async () => {
        const { result } = renderHook(() => useSubscribe(), { wrapper: createWrapper() });

        await act(async () => {
            await result.current.subscribe({ planName: 'Pro', amountXlm: 50 });
        });

        expect(callContract).toHaveBeenCalledWith({
            contractId: 'SUBSCRIPTION_MANAGER_ID',
            method: 'subscribe',
            source: ADDR,
            args: [address(ADDR), str('Pro'), i128(500_000_000n)],
        });
    });

    it('useSetConsent sends the four flags and a zero expiry by default', async () => {
        const { result } = renderHook(() => useSetConsent(), { wrapper: createWrapper() });

        act(() => {
            result.current.setConsent({
                dataProcessing: true,
                targetedAds: false,
                analytics: true,
                thirdPartySharing: false,
            });
        });

        await waitFor(() => expect(callContract).toHaveBeenCalled());
        expect(callContract).toHaveBeenCalledWith({
            contractId: 'PRIVACY_LAYER_ID',
            method: 'set_consent',
            source: ADDR,
            args: [address(ADDR), bool(true), bool(false), bool(true), bool(false), u64(0)],
        });
    });

    it('useCreateCampaign sends every argument in contract order', async () => {
        const { result } = renderHook(() => useCreateCampaign(), { wrapper: createWrapper() });

        await act(async () => {
            await result.current.createCampaign({
                campaignType: 2,
                budgetXlm: 100,
                costPerViewXlm: 0.5,
                durationDays: 3,
                targetViews: 1_000,
                dailyViewLimit: 50,
                refundable: false,
            });
        });

        expect(callContract).toHaveBeenCalledWith({
            contractId: 'CAMPAIGN_CONTRACT_ID',
            method: 'create_campaign',
            source: ADDR,
            args: [
                address(ADDR),
                u32(2),
                i128(1_000_000_000n),
                i128(5_000_000n),
                u32(3 * 17_280),
                u64(1_000),
                u64(50),
                bool(false),
            ],
        });
    });

    // Each entry renders the hook and returns a thunk that submits with it.
    const noWalletCases: [string, () => () => Promise<unknown>][] = [
        ['usePlaceBid', () => {
            const { placeBid } = usePlaceBid();
            return () => placeBid({ auctionId: 1, amountStroops: 1n, campaignId: 1 });
        }],
        ['useCastVote', () => {
            const { castVote } = useCastVote();
            return () => castVote({ proposalId: 1, voteType: 'For', votePower: 1n });
        }],
        ['useCreateProposal', () => {
            const { createProposal } = useCreateProposal();
            return () => createProposal({ title: 't', description: 'd', votingPeriodDays: 1 });
        }],
        ['useCreateAuction', () => {
            const { createAuction } = useCreateAuction();
            return () =>
                createAuction({ impressionSlot: 's', floorPriceXlm: 1, reservePriceXlm: 1, durationHours: 1 });
        }],
        ['useSubscribe', () => {
            const { subscribe } = useSubscribe();
            return () => subscribe({ planName: 'Pro', amountXlm: 1 });
        }],
    ];

    it.each(noWalletCases)('%s rejects without a connected wallet', async (_name, useSubmit) => {
        useWalletStore.getState().setAddress(null);
        const { result } = renderHook(() => useSubmit(), { wrapper: createWrapper() });

        await expect(result.current()).rejects.toThrow('Wallet not connected');
        expect(callContract).not.toHaveBeenCalled();
    });
});

describe('useWallet', () => {
    it('exposes the wallet store address and connection state', () => {
        useWalletStore.getState().setAddress(ADDR);
        useWalletStore.getState().setConnected(true);
        const { result } = renderHook(() => useWallet());
        expect(result.current).toEqual({ address: ADDR, isConnected: true });
    });
});
