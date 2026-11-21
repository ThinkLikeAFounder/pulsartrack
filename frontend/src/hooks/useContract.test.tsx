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
