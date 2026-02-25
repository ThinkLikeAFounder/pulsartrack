'use client';

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  callContract,
  callReadOnly,
  ContractCallOptions,
  ReadOnlyOptions,
} from '../lib/soroban-client';
import { CONTRACT_IDS } from '../lib/stellar-config';
import { useWalletStore } from '../store/wallet-store';
import {
  u64ToScVal,
  stringToScVal,
  i128ToScVal,
  addressToScVal,
  boolToScVal,
} from '../lib/soroban-client';

/**
 * Hook for contract write operations
 */
export function useContractCall() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (options: ContractCallOptions) => {
      return await callContract(options);
    },
    onSuccess: (data, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['contract', variables.contractId],
      });
    },
  });
}

/**
 * Hook for contract read-only operations
 */
export function useContractRead<T = any>(
  options: ReadOnlyOptions,
  enabled = true,
) {
  return useQuery<T, Error>({
    queryKey: ['contract', options.contractId, options.method, options.args],
    queryFn: () => callReadOnly(options),
    enabled: enabled && !!options.contractId,
    staleTime: 30_000,
    refetchInterval: 60_000,
  });
}

/**
 * Hook to get campaign details
 */
export function useCampaign(campaignId: number, enabled = true) {
  return useContractRead(
    {
      contractId: CONTRACT_IDS.CAMPAIGN_ORCHESTRATOR,
      method: 'get_campaign',
      args: [u64ToScVal(campaignId)],
    },
    enabled && campaignId > 0,
  );
}

/**
 * Hook to get publisher reputation
 */
export function usePublisherReputation(
  publisherAddress: string,
  enabled = true,
) {
  return useContractRead(
    {
      contractId: CONTRACT_IDS.PUBLISHER_REPUTATION,
      method: 'get_reputation',
      args: publisherAddress ? [addressToScVal(publisherAddress)] : [],
    },
    enabled && !!publisherAddress,
  );
}

/**
 * Hook to get advertiser stats
 */
export function useAdvertiserStats(advertiserAddress: string, enabled = true) {
  return useContractRead(
    {
      contractId: CONTRACT_IDS.CAMPAIGN_ORCHESTRATOR,
      method: 'get_advertiser_stats',
      args: advertiserAddress ? [addressToScVal(advertiserAddress)] : [],
    },
    enabled && !!advertiserAddress,
  );
}

/**
 * Hook to get campaign count
 */
export function useCampaignCount(enabled = true) {
  return useContractRead<number>(
    {
      contractId: CONTRACT_IDS.CAMPAIGN_ORCHESTRATOR,
      method: 'get_campaign_count',
      args: [],
    },
    enabled,
  );
}

/**
 * Hook to get all campaigns for an advertiser
 */
export function useAdvertiserCampaigns(
  advertiserAddress: string,
  campaignCount: number | undefined,
  enabled = true,
) {
  return useQuery({
    queryKey: ['advertiser_campaigns', advertiserAddress, campaignCount],
    queryFn: async () => {
      if (!campaignCount) return [];
      const campaigns: any[] = [];
      // Fetch concurrently for better performance
      const promises = [];
      for (let i = 1; i <= campaignCount; i++) {
        promises.push(
          callReadOnly({
            contractId: CONTRACT_IDS.CAMPAIGN_ORCHESTRATOR,
            method: 'get_campaign',
            args: [u64ToScVal(i)],
          })
            .then((campaign) => {
              if (campaign && campaign.advertiser === advertiserAddress) {
                campaigns.push({ id: i, ...campaign });
              }
            })
            .catch(() => null), // Ignore missing or failed campaigns
        );
      }
      await Promise.all(promises);
      return campaigns.sort((a, b) => Number(b.id) - Number(a.id));
    },
    enabled: enabled && !!advertiserAddress && (campaignCount ?? 0) > 0,
  });
}

/**
 * Hook to get subscription status
 */
export function useSubscription(subscriberAddress: string, enabled = true) {
  return useContractRead(
    {
      contractId: CONTRACT_IDS.SUBSCRIPTION_MANAGER,
      method: 'get_subscription',
      args: subscriberAddress ? [addressToScVal(subscriberAddress)] : [],
    },
    enabled && !!subscriberAddress,
  );
}

/**
 * Hook to get privacy consent
 */
export function usePrivacyConsent(userAddress: string, enabled = true) {
  return useContractRead(
    {
      contractId: CONTRACT_IDS.PRIVACY_LAYER,
      method: 'get_consent',
      args: userAddress ? [addressToScVal(userAddress)] : [],
    },
    enabled && !!userAddress,
  );
}

/**
 * Hook to get auction details
 */
export function useAuction(auctionId: number, enabled = true) {
  return useContractRead(
    {
      contractId: CONTRACT_IDS.AUCTION_ENGINE,
      method: 'get_auction',
      args: [u64ToScVal(auctionId)],
    },
    enabled && auctionId > 0,
  );
}

/**
 * Hook to create a campaign
 */
export function useCreateCampaign() {
  const { mutateAsync, ...rest } = useContractCall();
  const { address } = useWalletStore();

  const createCampaign = async (params: {
    title: string;
    budgetXlm: number;
    dailyBudgetXlm: number;
    durationDays: number;
    contentId: string;
  }) => {
    if (!address) throw new Error('Wallet not connected');
    const STROOPS = 10_000_000;
    return mutateAsync({
      contractId: CONTRACT_IDS.CAMPAIGN_ORCHESTRATOR,
      method: 'create_campaign',
      source: address,
      args: [
        addressToScVal(address),
        stringToScVal(params.title),
        i128ToScVal(Math.floor(params.budgetXlm * STROOPS)),
        i128ToScVal(Math.floor(params.dailyBudgetXlm * STROOPS)),
        u64ToScVal(params.durationDays * 86400),
        stringToScVal(params.contentId),
      ],
    });
  };

  return { createCampaign, ...rest };
}

/**
 * Hook to place a bid in an auction
 */
export function usePlaceBid() {
  const { mutateAsync, ...rest } = useContractCall();
  const { address } = useWalletStore();

  const placeBid = async (params: {
    auctionId: number;
    amountStroops: bigint;
    campaignId: number;
  }) => {
    if (!address) throw new Error('Wallet not connected');
    return mutateAsync({
      contractId: CONTRACT_IDS.AUCTION_ENGINE,
      method: 'place_bid',
      source: address,
      args: [
        addressToScVal(address),
        u64ToScVal(params.auctionId),
        i128ToScVal(params.amountStroops),
        u64ToScVal(params.campaignId),
      ],
    });
  };

  return { placeBid, ...rest };
}

/**
 * Hook to set privacy consent
 */
export function useSetConsent() {
  const { mutate, ...rest } = useContractCall();
  const { address } = useWalletStore();

  const setConsent = (params: {
    dataProcessing: boolean;
    targetedAds: boolean;
    analytics: boolean;
    thirdPartySharing: boolean;
    expiresInDays?: number;
  }) => {
    if (!address) return;
    mutate({
      contractId: CONTRACT_IDS.PRIVACY_LAYER,
      method: 'set_consent',
      source: address,
      args: [
        addressToScVal(address),
        boolToScVal(params.dataProcessing),
        boolToScVal(params.targetedAds),
        boolToScVal(params.analytics),
        boolToScVal(params.thirdPartySharing),
      ],
    });
  };

  return { setConsent, ...rest };
}

/**
 * Hook to get PULSAR token balance
 */
export function useGovernanceBalance(userAddress: string, enabled = true) {
  return useContractRead<bigint>(
    {
      contractId: CONTRACT_IDS.GOVERNANCE_TOKEN,
      method: 'balance',
      args: userAddress ? [addressToScVal(userAddress)] : [],
    },
    enabled && !!userAddress,
  );
}

/**
 * Hook to get governance proposal count
 */
export function useProposalCount(enabled = true) {
  return useContractRead<number>(
    {
      contractId: CONTRACT_IDS.GOVERNANCE_DAO,
      method: 'get_proposal_count',
      args: [],
    },
    enabled,
  );
}

/**
 * Hook to get a specific governance proposal
 */
export function useGovernanceProposal(proposalId: number, enabled = true) {
  return useContractRead(
    {
      contractId: CONTRACT_IDS.GOVERNANCE_DAO,
      method: 'get_proposal',
      args: [u64ToScVal(proposalId)],
    },
    enabled && proposalId > 0,
  );
}

/**
 * Hook to get all governance proposals
 */
export function useGovernanceProposals(
  proposalCount: number | undefined,
  enabled = true,
) {
  return useQuery({
    queryKey: ['governance_proposals', proposalCount],
    queryFn: async () => {
      if (!proposalCount || proposalCount === 0) return [];
      const proposals: any[] = [];
      const promises = [];
      for (let i = 1; i <= proposalCount; i++) {
        promises.push(
          callReadOnly({
            contractId: CONTRACT_IDS.GOVERNANCE_DAO,
            method: 'get_proposal',
            args: [u64ToScVal(i)],
          })
            .then((proposal) => {
              if (proposal) {
                proposals.push({ id: i, ...proposal });
              }
            })
            .catch(() => null),
        );
      }
      await Promise.all(promises);
      return proposals.sort((a, b) => Number(b.id) - Number(a.id));
    },
    enabled: enabled && (proposalCount ?? 0) > 0,
  });
}

/**
 * Hook to cast a vote on a governance proposal
 */
export function useCastVote() {
  const { mutateAsync, ...rest } = useContractCall();
  const { address } = useWalletStore();

  const castVote = async (params: {
    proposalId: number;
    voteType: 'For' | 'Against' | 'Abstain';
    votePower: bigint;
  }) => {
    if (!address) throw new Error('Wallet not connected');
    return mutateAsync({
      contractId: CONTRACT_IDS.GOVERNANCE_DAO,
      method: 'cast_vote',
      source: address,
      args: [
        addressToScVal(address),
        u64ToScVal(params.proposalId),
        stringToScVal(params.voteType),
        i128ToScVal(params.votePower),
      ],
    });
  };

  return { castVote, ...rest };
}

/**
 * Hook to create a governance proposal
 */
export function useCreateProposal() {
  const { mutateAsync, ...rest } = useContractCall();
  const { address } = useWalletStore();

  const createProposal = async (params: {
    title: string;
    description: string;
    votingPeriodDays: number;
  }) => {
    if (!address) throw new Error('Wallet not connected');
    return mutateAsync({
      contractId: CONTRACT_IDS.GOVERNANCE_DAO,
      method: 'create_proposal',
      source: address,
      args: [
        addressToScVal(address),
        stringToScVal(params.title),
        stringToScVal(params.description),
        u64ToScVal(params.votingPeriodDays * 86400),
      ],
    });
  };

  return { createProposal, ...rest };
}

/**
 * Hook to get publisher info
 */
export function usePublisherData(publisherAddress: string, enabled = true) {
  return useContractRead(
    {
      contractId: CONTRACT_IDS.PUBLISHER_VERIFICATION,
      method: 'get_publisher',
      args: publisherAddress ? [addressToScVal(publisherAddress)] : [],
    },
    enabled && !!publisherAddress,
  );
}

/**
 * Hook to get KYC record for publisher
 */
export function usePublisherKyc(publisherAddress: string, enabled = true) {
  return useContractRead(
    {
      contractId: CONTRACT_IDS.PUBLISHER_VERIFICATION,
      method: 'get_kyc_record',
      args: publisherAddress ? [addressToScVal(publisherAddress)] : [],
    },
    enabled && !!publisherAddress,
  );
}

/**
 * Hook to get publisher earnings
 */
export function usePublisherEarnings(publisherAddress: string, enabled = true) {
  return useContractRead<bigint>(
    {
      contractId: CONTRACT_IDS.REVENUE_SETTLEMENT,
      method: 'get_publisher_balance',
      args: publisherAddress ? [addressToScVal(publisherAddress)] : [],
    },
    enabled && !!publisherAddress,
  );
}

/**
 * Hook to get active auctions count
 */
export function useAuctionCount(enabled = true) {
  return useContractRead<number>(
    {
      contractId: CONTRACT_IDS.AUCTION_ENGINE,
      method: 'get_auction_count',
      args: [],
    },
    enabled,
  );
}

/**
 * Hook to get all auctions for a publisher
 */
export function usePublisherAuctions(
  publisherAddress: string,
  auctionCount: number | undefined,
  enabled = true,
) {
  return useQuery({
    queryKey: ['publisher_auctions', publisherAddress, auctionCount],
    queryFn: async () => {
      if (!auctionCount || auctionCount === 0) return [];
      const auctions: any[] = [];
      const promises = [];
      for (let i = 1; i <= auctionCount; i++) {
        promises.push(
          callReadOnly({
            contractId: CONTRACT_IDS.AUCTION_ENGINE,
            method: 'get_auction',
            args: [u64ToScVal(i)],
          })
            .then((auction) => {
              if (auction && auction.publisher === publisherAddress) {
                auctions.push({ id: i, ...auction });
              }
            })
            .catch(() => null),
        );
      }
      await Promise.all(promises);
      return auctions.sort((a, b) => Number(b.id) - Number(a.id));
    },
    enabled: enabled && !!publisherAddress && (auctionCount ?? 0) > 0,
  });
}

/**
 * Hook to subscribe to a plan
 */
export function useSubscribe() {
  const { mutateAsync, ...rest } = useContractCall();
  const { address } = useWalletStore();

  const subscribe = async (params: { planName: string; amountXlm: number }) => {
    if (!address) throw new Error('Wallet not connected');
    const STROOPS = 10_000_000;
    return mutateAsync({
      contractId: CONTRACT_IDS.SUBSCRIPTION_MANAGER,
      method: 'subscribe',
      source: address,
      args: [
        addressToScVal(address),
        stringToScVal(params.planName),
        i128ToScVal(Math.floor(params.amountXlm * STROOPS)),
      ],
    });
  };

  return { subscribe, ...rest };
}

/**
 * Convenience hook for wallet info
 */
export function useWallet() {
  const { address, isConnected } = useWalletStore();
  return { address, isConnected };
}
