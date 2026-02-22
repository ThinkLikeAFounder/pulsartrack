'use client';

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { callContract, callReadOnly, ContractCallOptions, ReadOnlyOptions } from '../lib/soroban-client';
import { CONTRACT_IDS } from '../lib/stellar-config';
import { useWalletStore } from '../store/wallet-store';
import { u64ToScVal, stringToScVal, i128ToScVal, addressToScVal, boolToScVal } from '../lib/soroban-client';

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
      queryClient.invalidateQueries({ queryKey: ['contract', variables.contractId] });
    },
  });
}

/**
 * Hook for contract read-only operations
 */
export function useContractRead<T = any>(options: ReadOnlyOptions, enabled = true) {
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
    enabled && campaignId > 0
  );
}

/**
 * Hook to get publisher reputation
 */
export function usePublisherReputation(publisherAddress: string, enabled = true) {
  return useContractRead(
    {
      contractId: CONTRACT_IDS.PUBLISHER_REPUTATION,
      method: 'get_reputation',
      args: publisherAddress ? [addressToScVal(publisherAddress)] : [],
    },
    enabled && !!publisherAddress
  );
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
    enabled && !!subscriberAddress
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
    enabled && !!userAddress
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
    enabled && auctionId > 0
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
    if (!address) throw new Error("Wallet not connected");
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
    if (!address) throw new Error("Wallet not connected");
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
 * Convenience hook for wallet info
 */
export function useWallet() {
  const { address, isConnected } = useWalletStore();
  return { address, isConnected };
}
