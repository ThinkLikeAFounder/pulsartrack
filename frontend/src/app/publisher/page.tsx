'use client';

import { useState } from 'react';
import {
  Radio,
  DollarSign,
  Star,
  TrendingUp,
  Clock,
  CheckCircle,
} from 'lucide-react';
import { useWalletStore } from '@/store/wallet-store';
import { WalletConnectButton } from '@/components/wallet/WalletModal';
import {
  usePublisherReputation,
  usePublisherData,
  usePublisherKyc,
  usePublisherEarnings,
  useAuctionCount,
  usePublisherAuctions,
  useCreateAuction,
  useSubscription,
  useSubscribe,
} from '@/hooks/useContract';
import { formatAddress } from '@/lib/display-utils';
import { formatXlm } from '@/lib/display-utils';
import { SubscriptionStatus } from '@/components/subscription/SubscriptionStatus';
import { PlanCard } from '@/components/subscription/PlanCard';
import { PublisherCard } from '@/components/publisher/PublisherCard';
import { EarningsDisplay } from '@/components/publisher/EarningsDisplay';
import { toPublisher } from '@/lib/contract-adapters';
import type { Subscription, SubscriptionPlan, SubscriptionTier } from '@/types/contracts';

type PublisherTab = 'overview' | 'auctions' | 'earnings' | 'subscription';

interface PublisherReputation {
  score?: number;
}

interface PublisherInfo {
  status?: unknown;
  impressions_served?: number | string;
}

interface PublisherKyc {
  verified?: boolean;
}

const SUBSCRIPTION_PLANS = [
  {
    name: 'Starter',
    priceMonthly: '99 XLM',
    priceAnnual: '990 XLM',
    features: [
      'Up to 5 campaigns',
      '100K impressions/mo',
      '10 publishers',
      'Basic analytics',
    ],
    color: 'gray',
  },
  {
    name: 'Growth',
    priceMonthly: '299 XLM',
    priceAnnual: '2,990 XLM',
    features: [
      'Up to 25 campaigns',
      '500K impressions/mo',
      '50 publishers',
      'Full analytics',
    ],
    color: 'blue',
    popular: true,
  },
  {
    name: 'Business',
    priceMonthly: '799 XLM',
    priceAnnual: '7,990 XLM',
    features: [
      'Up to 100 campaigns',
      '2M impressions/mo',
      '200 publishers',
      'Analytics + API',
    ],
    color: 'purple',
  },
  {
    name: 'Enterprise',
    priceMonthly: '1,999 XLM',
    priceAnnual: '19,990 XLM',
    features: [
      'Unlimited campaigns',
      '10M impressions/mo',
      '1000 publishers',
      'Analytics + API',
    ],
    color: 'indigo',
  },
];

import { ErrorBoundary } from '@/components/ErrorBoundary';

const STROOPS_PER_XLM = BigInt(10_000_000);

/** The plans above in the shape `PlanCard` renders. */
const PLAN_CARDS: Array<SubscriptionPlan & { features: string[] }> = SUBSCRIPTION_PLANS.map((plan) => ({
  tier: plan.name as SubscriptionTier,
  name: plan.name,
  price_monthly: BigInt(parseInt(plan.priceMonthly.replace(/,/g, ''), 10)) * STROOPS_PER_XLM,
  price_annual: BigInt(parseInt(plan.priceAnnual.replace(/,/g, ''), 10)) * STROOPS_PER_XLM,
  max_campaigns: 0,
  max_impressions_per_month: BigInt(0),
  max_publishers: 0,
  analytics_enabled: true,
  api_access: plan.features.some((f) => f.includes('API')),
  features: plan.features,
}));

export default function PublisherPage() {
  const { address, isConnected } = useWalletStore();
  const [activeTab, setActiveTab] = useState<PublisherTab>('overview');

  // Fetch publisher data
  const { data: reputation } = usePublisherReputation(address || '') as {
    data: PublisherReputation | undefined;
  };
  const { data: publisherData, isLoading: publisherLoading } = usePublisherData(
    address || '',
    isConnected,
  ) as { data: PublisherInfo | undefined; isLoading: boolean };
  const { data: kycData, isLoading: kycLoading } = usePublisherKyc(
    address || '',
    isConnected,
  ) as { data: PublisherKyc | undefined; isLoading: boolean };
  const { data: earnings, isLoading: earningsLoading } = usePublisherEarnings(
    address || '',
    isConnected,
  );
  const { data: auctionCount } = useAuctionCount(isConnected);
  const { data: auctions, isLoading: auctionsLoading } = usePublisherAuctions(
    address || '',
    auctionCount,
    isConnected,
  );
  const { data: subscriptionData } = useSubscription(
    address || '',
    isConnected,
  ) as { data: Subscription | null | undefined };
  const { subscribe, isPending: subscribeLoading } = useSubscribe();
  const { createAuction, isPending: creatingAuction } = useCreateAuction();
  const [showCreateAuction, setShowCreateAuction] = useState(false);
  const [auctionForm, setAuctionForm] = useState({
    impressionSlot: '',
    floorPriceXlm: '1',
    reservePriceXlm: '0.5',
    durationHours: '24',
  });
  const [auctionFormError, setAuctionFormError] = useState<string | null>(null);

  if (!isConnected) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center max-w-md p-8 bg-white rounded-2xl shadow-sm border border-gray-200">
          <div className="w-16 h-16 bg-indigo-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <Radio className="w-8 h-8 text-indigo-600" />
          </div>
          <h2 className="text-2xl font-bold text-gray-900 mb-2">
            Publisher Dashboard
          </h2>
          <p className="text-gray-600 mb-6">
            Connect your Freighter wallet to earn XLM by serving ads on the
            Stellar network.
          </p>
          <WalletConnectButton />
        </div>
      </div>
    );
  }

  const reputationScore = reputation ? (reputation.score ?? 500) : 500;

  return (
    <ErrorBoundary name="PublisherPage" resetKeys={[activeTab]}>
      <div className="min-h-screen bg-gray-50">
        {/* Page Header */}
        <div className="bg-white border-b border-gray-200 px-4 sm:px-6 lg:px-8 py-6">
          <div className="max-w-7xl mx-auto flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">
                Publisher Dashboard
              </h1>
              <p className="text-sm text-gray-500 mt-1 font-mono">
                {formatAddress(address || '')}
              </p>
            </div>
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-1 px-3 py-1 bg-amber-100 text-amber-700 rounded-full text-sm font-medium">
                <Star className="w-3.5 h-3.5" />
                Rep: {reputation ? (reputation.score ?? 500) : 500}
                /1000
              </div>
              <span className="px-3 py-1 bg-green-100 text-green-700 rounded-full text-sm font-medium">
                Stellar Testnet
              </span>
            </div>
          </div>
        </div>

        {/* Stats */}
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
            {[
              {
                icon: DollarSign,
                label: 'Total Earned',
                value: earningsLoading
                  ? 'Loading...'
                  : `${formatXlm(earnings || 0n)}`,
                bgClass: 'bg-green-100',
                iconClass: 'text-green-600',
              },
              {
                icon: TrendingUp,
                label: 'Impressions Served',
                value: publisherLoading
                  ? 'Loading...'
                  : (publisherData?.impressions_served ?? '0'),
                bgClass: 'bg-blue-100',
                iconClass: 'text-blue-600',
              },
              {
                icon: Star,
                label: 'Reputation Score',
                value: `${reputation ? (reputation.score ?? 500) : 500}`,
                bgClass: 'bg-amber-100',
                iconClass: 'text-amber-600',
              },
              {
                icon: Clock,
                label: 'Active Auctions',
                value: auctionsLoading ? 'Loading...' : (auctions?.length ?? 0),
                bgClass: 'bg-purple-100',
                iconClass: 'text-purple-600',
              },
            ].map(({ icon: Icon, label, value, bgClass, iconClass }) => (
              <div
                key={label}
                className="bg-white p-4 rounded-xl border border-gray-200"
              >
                <div className="flex items-center gap-3">
                  <div
                    className={`w-10 h-10 ${bgClass} rounded-lg flex items-center justify-center`}
                  >
                    <Icon className={`w-5 h-5 ${iconClass}`} />
                  </div>
                  <div>
                    <p className="text-sm text-gray-600">{label}</p>
                    <p className="text-xl font-bold text-gray-900">{value}</p>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Tabs */}
          <div className="flex gap-1 mb-6 bg-gray-100 p-1 rounded-lg w-fit flex-wrap">
            {([
              { id: 'overview', label: 'Overview' },
              { id: 'auctions', label: 'RTB Auctions' },
              { id: 'earnings', label: 'Earnings' },
              { id: 'subscription', label: 'Subscription Plans' },
            ] as Array<{ id: PublisherTab; label: string }>).map(({ id, label }) => (
              <button
                key={id}
                onClick={() => setActiveTab(id)}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  activeTab === id
                    ? 'bg-white text-indigo-600 shadow-sm'
                    : 'text-gray-600 hover:text-gray-900'
                }`}
              >
                {label}
              </button>
            ))}
          </div>

          {activeTab === 'overview' && publisherData?.status !== undefined && address && (
            <div className="mb-6 max-w-md">
              <PublisherCard publisher={toPublisher(address, publisherData)} />
            </div>
          )}

          {activeTab === 'overview' && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="bg-white rounded-xl border border-gray-200 p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">
                  Verification Status
                </h3>
                <div className="space-y-3">
                  {[
                    {
                      label: 'Wallet Connected',
                      done: isConnected,
                    },
                    {
                      label: 'Publisher Registered',
                      done: publisherLoading
                        ? null
                        : !!(publisherData && publisherData.status),
                    },
                    {
                      label: 'KYC Verified',
                      done: kycLoading
                        ? null
                        : !!(kycData && kycData.verified),
                    },
                    {
                      label: 'Reputation Initialized',
                      done: !!(reputation && reputation.score),
                    },
                  ].map(({ label, done }) => (
                    <div key={label} className="flex items-center gap-3">
                      <CheckCircle
                        className={`w-5 h-5 ${
                          done === null
                            ? 'text-gray-300'
                            : done
                              ? 'text-green-500'
                              : 'text-gray-300'
                        }`}
                      />
                      <span
                        className={`text-sm ${
                          done === null
                            ? 'text-gray-500'
                            : done
                              ? 'text-gray-900'
                              : 'text-gray-500'
                        }`}
                      >
                        {done === null && 'Loading... '}
                        {label}
                      </span>
                    </div>
                  ))}
                </div>
              </div>

              <div className="bg-white rounded-xl border border-gray-200 p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">
                  Publisher Tiers
                </h3>
                <div className="space-y-2">
                  {/* Issue #370 — Platinum uses Infinity as upper bound so scores
                      above 1000 still resolve to a tier. Negative scores are clamped
                      to 0 before matching so they always land in Bronze. */}
                  {([
                    { tier: 'Bronze',   min: 0,   max: 399,      color: 'amber'  },
                    { tier: 'Silver',   min: 400,  max: 599,      color: 'gray'   },
                    { tier: 'Gold',     min: 600,  max: 799,      color: 'yellow' },
                    { tier: 'Platinum', min: 800,  max: Infinity, color: 'blue'   },
                  ] as Array<{ tier: string; min: number; max: number; color: string }>).map(({ tier, min, max }) => {
                    const score = Math.max(reputationScore, 0);
                    const isActive = score >= min && score <= max;
                    const rangeLabel =
                      max === Infinity ? `${min}+ score` : `${min} – ${max} score`;
                    return (
                      <div
                        key={tier}
                        className={`flex items-center justify-between px-3 py-2 rounded-lg ${
                          isActive
                            ? 'bg-indigo-50 border border-indigo-200'
                            : 'bg-gray-50'
                        }`}
                      >
                        <span className="text-sm font-medium text-gray-900">
                          {tier}
                        </span>
                        <span className="text-xs text-gray-500">{rangeLabel}</span>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          )}

          {activeTab === 'auctions' && (
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-lg font-semibold text-gray-900">
                  Real-Time Bidding Auctions
                </h2>
                {!showCreateAuction && (
                  <button
                    onClick={() => setShowCreateAuction(true)}
                    className="px-3 py-1.5 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors text-sm"
                  >
                    Create Impression Slot
                  </button>
                )}
              </div>

              {showCreateAuction && (
                <form
                  onSubmit={async (e) => {
                    e.preventDefault();
                    setAuctionFormError(null);
                    if (!auctionForm.impressionSlot.trim()) {
                      setAuctionFormError('Impression slot name is required');
                      return;
                    }
                    try {
                      await createAuction({
                        impressionSlot: auctionForm.impressionSlot,
                        floorPriceXlm: parseFloat(auctionForm.floorPriceXlm) || 0,
                        reservePriceXlm: parseFloat(auctionForm.reservePriceXlm) || 0,
                        durationHours: parseFloat(auctionForm.durationHours) || 24,
                      });
                      setShowCreateAuction(false);
                      setAuctionForm({
                        impressionSlot: '',
                        floorPriceXlm: '1',
                        reservePriceXlm: '0.5',
                        durationHours: '24',
                      });
                    } catch (err) {
                      setAuctionFormError(
                        err instanceof Error ? err.message : 'Failed to create auction',
                      );
                    }
                  }}
                  className="mb-6 p-4 border border-gray-200 rounded-lg bg-gray-50 space-y-3"
                >
                  <div>
                    <label htmlFor="auction-slot-name" className="block text-sm font-medium text-gray-700 mb-1">
                      Impression Slot Name
                    </label>
                    <input
                      id="auction-slot-name"
                      type="text"
                      value={auctionForm.impressionSlot}
                      onChange={(e) =>
                        setAuctionForm({ ...auctionForm, impressionSlot: e.target.value })
                      }
                      placeholder="homepage-hero-banner"
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                      required
                    />
                  </div>
                  <div className="grid grid-cols-3 gap-3">
                    <div>
                      <label htmlFor="auction-floor-price" className="block text-sm font-medium text-gray-700 mb-1">
                        Floor (XLM)
                      </label>
                      <input
                        id="auction-floor-price"
                        type="number"
                        min="0.0001"
                        step="0.0001"
                        value={auctionForm.floorPriceXlm}
                        onChange={(e) =>
                          setAuctionForm({ ...auctionForm, floorPriceXlm: e.target.value })
                        }
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                        required
                      />
                    </div>
                    <div>
                      <label htmlFor="auction-reserve-price" className="block text-sm font-medium text-gray-700 mb-1">
                        Reserve (XLM)
                      </label>
                      <input
                        id="auction-reserve-price"
                        type="number"
                        min="0"
                        step="0.0001"
                        value={auctionForm.reservePriceXlm}
                        onChange={(e) =>
                          setAuctionForm({ ...auctionForm, reservePriceXlm: e.target.value })
                        }
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                        required
                      />
                    </div>
                    <div>
                      <label htmlFor="auction-duration" className="block text-sm font-medium text-gray-700 mb-1">
                        Duration (hours)
                      </label>
                      <input
                        id="auction-duration"
                        type="number"
                        min="1"
                        max="720"
                        value={auctionForm.durationHours}
                        onChange={(e) =>
                          setAuctionForm({ ...auctionForm, durationHours: e.target.value })
                        }
                        className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                        required
                      />
                    </div>
                  </div>
                  {auctionFormError && (
                    <div className="p-2 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
                      {auctionFormError}
                    </div>
                  )}
                  <div className="flex gap-2">
                    <button
                      type="submit"
                      disabled={creatingAuction}
                      className="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:opacity-50 transition-colors text-sm font-medium"
                    >
                      {creatingAuction ? 'Listing...' : 'List Impression Slot'}
                    </button>
                    <button
                      type="button"
                      onClick={() => setShowCreateAuction(false)}
                      className="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors text-sm"
                    >
                      Cancel
                    </button>
                  </div>
                </form>
              )}

              {auctionsLoading ? (
                <div className="text-center py-12 text-gray-500">
                  <p>Loading auctions...</p>
                </div>
              ) : auctions && auctions.length > 0 ? (
                <div className="space-y-4">
                  {auctions.map((auction) => (
                    <div
                      key={auction.id}
                      className="border border-gray-200 rounded-lg p-4"
                    >
                      <div className="flex items-start justify-between mb-2">
                        <h3 className="font-semibold text-gray-900">
                          Auction #{auction.id}
                        </h3>
                        <span className="px-2 py-1 bg-blue-100 text-blue-700 text-xs rounded-full">
                          {auction.status || 'Active'}
                        </span>
                      </div>
                      <p className="text-sm text-gray-600 mb-2">
                        Current Bid:{' '}
                        {formatXlm(auction.current_bid || 0n, 4)}
                      </p>
                      <p className="text-sm text-gray-500">
                        Ends:{' '}
                        {new Date(
                          Number(auction.end_time || 0) * 1000,
                        ).toLocaleString()}
                      </p>
                    </div>
                  ))}
                </div>
              ) : (
                !showCreateAuction && (
                  <div className="text-center py-12 text-gray-500">
                    <Clock className="w-12 h-12 mx-auto mb-3 opacity-30" />
                    <p>
                      No active auctions. Create an impression slot to start
                      receiving bids.
                    </p>
                    <button
                      onClick={() => setShowCreateAuction(true)}
                      className="mt-4 px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors text-sm"
                    >
                      Create Impression Slot
                    </button>
                  </div>
                )
              )}
            </div>
          )}

          {activeTab === 'earnings' && (
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <h2 className="text-lg font-semibold text-gray-900 mb-4">
                XLM Earnings
              </h2>
              {earningsLoading ? (
                <div className="text-center py-12 text-gray-500">
                  <p>Loading earnings...</p>
                </div>
              ) : (earnings ?? 0n) > 0n ? (
                <div className="bg-gray-900 rounded-xl p-4">
                  <EarningsDisplay
                    totalEarnings={
                      publisherData && address
                        ? toPublisher(address, publisherData).earnings_total
                        : earnings || 0n
                    }
                    pendingPayout={earnings || 0n}
                  />
                </div>
              ) : (
                <div className="text-center py-12 text-gray-500">
                  <DollarSign className="w-12 h-12 mx-auto mb-3 opacity-30" />
                  <p>No earnings yet. Start serving ads to earn XLM.</p>
                </div>
              )}
            </div>
          )}

          {activeTab === 'subscription' && (
            <div className="space-y-6">
              <div className="max-w-md">
                <h2 className="text-lg font-semibold text-gray-900 mb-3">
                  Current Subscription
                </h2>
                <SubscriptionStatus
                  subscription={subscriptionData ?? null}
                  onRenew={() => {
                    if (!subscriptionData) return;
                    const plan = SUBSCRIPTION_PLANS.find(
                      (p) => p.name === subscriptionData.tier,
                    );
                    const monthlyPrice = plan
                      ? parseInt(plan.priceMonthly.split(' ')[0])
                      : 99;
                    subscribe({
                      planName: subscriptionData.tier,
                      amountXlm: monthlyPrice,
                    });
                  }}
                />
              </div>
              <div>
                <h2 className="text-lg font-semibold text-gray-900 mb-6">
                  Subscription Plans
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                {PLAN_CARDS.map((plan) => (
                  <PlanCard
                    key={plan.tier}
                    plan={plan}
                    isCurrentPlan={subscriptionData?.tier === plan.tier}
                    isLoading={subscribeLoading}
                    onSubscribe={(chosen) =>
                      subscribe({
                        planName: chosen.name,
                        amountXlm: Number(chosen.price_monthly / STROOPS_PER_XLM),
                      })
                    }
                  />
                ))}
              </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </ErrorBoundary>
  );
}
