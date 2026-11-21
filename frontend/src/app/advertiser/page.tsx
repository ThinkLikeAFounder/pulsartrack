"use client";

import { useMemo, useState } from 'react';
import { PlusCircle, BarChart3, DollarSign, TrendingUp, Eye, MousePointer, Gavel, Target } from 'lucide-react';
import { useWalletStore } from '@/store/wallet-store';
import { WalletConnectButton } from '@/components/wallet/WalletModal';
import {
  useCampaignCount,
  useAdvertiserCampaigns,
  useAdvertiserStats,
  useAuctionCount,
  useOpenAuctions,
  AdvertiserCampaign,
} from '@/hooks/useContract';
import { formatXlm } from '@/lib/display-utils';
import { AuctionList } from '@/components/auction/AuctionList';
import { CampaignForm } from '@/components/campaign/CampaignForm';
import { CampaignCard } from '@/components/campaign/CampaignCard';
import { CampaignStats } from '@/components/campaign/CampaignStats';
import { AnalyticsDashboard } from '@/components/analytics/AnalyticsDashboard';
import { toCampaign } from '@/lib/contract-adapters';
import { TargetingForm, type TargetingConfig } from '@/components/targeting/TargetingForm';
import type { Auction } from '@/types/contracts';

type AdvertiserTab = 'campaigns' | 'create' | 'targeting' | 'auctions' | 'analytics';

import { ErrorBoundary } from "@/components/ErrorBoundary";

const COLOR_MAP = {
  blue: { bg: "bg-blue-100", text: "text-blue-600" },
  green: { bg: "bg-green-100", text: "text-green-600" },
  purple: { bg: "bg-purple-100", text: "text-purple-600" },
  orange: { bg: "bg-orange-100", text: "text-orange-600" },
};

export default function AdvertiserPage() {
  const { address, isConnected } = useWalletStore();

  const { data: stats, isLoading: isStatsLoading } = useAdvertiserStats(
    address as string,
  );
  const { data: campaignCount } = useCampaignCount();
  const { data: campaigns, isLoading: isCampaignsLoading } =
    useAdvertiserCampaigns(address as string, Number(campaignCount));
  const { data: auctionCount } = useAuctionCount();
  const { data: openAuctions, isLoading: isAuctionsLoading, refetch: refetchAuctions } =
    useOpenAuctions(auctionCount);

  const [activeTab, setActiveTab] = useState<AdvertiserTab>('campaigns');
  const [timeframe, setTimeframe] = useState<'7d' | '30d' | '90d'>('30d');
  const campaignViews = useMemo(
    () => ((campaigns ?? []) as AdvertiserCampaign[]).map(toCampaign),
    [campaigns],
  );
  const [createdCampaignId, setCreatedCampaignId] = useState<number | null>(null);
  const [targetingConfig, setTargetingConfig] = useState<Partial<TargetingConfig>>({});

  if (!isConnected) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center max-w-md p-8 bg-white rounded-2xl shadow-sm border border-gray-200">
          <div className="w-16 h-16 bg-indigo-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <DollarSign className="w-8 h-8 text-indigo-600" />
          </div>
          <h2 className="text-2xl font-bold text-gray-900 mb-2">
            Advertiser Dashboard
          </h2>
          <p className="text-gray-600 mb-6">
            Connect your Freighter wallet to manage campaigns on the Stellar
            network.
          </p>
          <WalletConnectButton />
        </div>
      </div>
    );
  }

  return (
    <ErrorBoundary name="AdvertiserPage" resetKeys={[activeTab]}>
      <div className="min-h-screen bg-gray-50">
        {/* Page Header */}
        <div className="bg-white border-b border-gray-200 px-4 sm:px-6 lg:px-8 py-6">
          <div className="max-w-7xl mx-auto flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">
                Advertiser Dashboard
              </h1>
              <p className="text-sm text-gray-500 mt-1 font-mono">{address}</p>
            </div>
            <div className="flex items-center gap-3">
              <span className="px-3 py-1 bg-green-100 text-green-700 rounded-full text-sm font-medium">
                Stellar Testnet
              </span>
            </div>
          </div>
        </div>

        {/* Stats Overview */}
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          {isStatsLoading ? (
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
              {[1, 2, 3, 4].map((idx) => (
                <div
                  key={idx}
                  className="bg-white p-4 rounded-xl border border-gray-200 animate-pulse"
                >
                  <div className="h-5 bg-gray-200 rounded w-1/2 mb-2"></div>
                  <div className="h-8 bg-gray-200 rounded w-1/4"></div>
                </div>
              ))}
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
              {[
                {
                  icon: BarChart3,
                  label: "Active Campaigns",
                  value: stats?.active_campaigns?.toString() || "0",
                  color: "blue" as const,
                },
                {
                  icon: Eye,
                  label: "Total Impressions",
                  value: stats?.total_views?.toString() || "0",
                  color: "green" as const,
                },
                {
                  icon: MousePointer,
                  label: "Total Clicks",
                  value: "0",
                  color: "purple" as const,
                },
                {
                  icon: TrendingUp,
                  label: "Total Spent",
                  value: stats ? formatXlm(stats.total_spent) : "0 XLM",
                  color: "orange" as const,
                },
              ].map(({ icon: Icon, label, value, color }) => (
                <div
                  key={label}
                  className="bg-white p-4 rounded-xl border border-gray-200"
                >
                  <div className="flex items-center gap-3">
                    <div
                      className={`w-10 h-10 ${COLOR_MAP[color].bg} rounded-lg flex items-center justify-center`}
                    >
                      <Icon className={`w-5 h-5 ${COLOR_MAP[color].text}`} />
                    </div>
                    <div>
                      <p className="text-sm text-gray-600">{label}</p>
                      <p className="text-xl font-bold text-gray-900">{value}</p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Tabs */}
          <div className="flex gap-1 mb-6 bg-gray-100 p-1 rounded-lg w-fit">
            {([
              { id: "campaigns", label: "My Campaigns", icon: BarChart3 },
              { id: "create", label: "Create Campaign", icon: PlusCircle },
              { id: "targeting", label: "Targeting", icon: Target },
              { id: "auctions", label: "RTB Auctions", icon: Gavel },
              { id: "analytics", label: "Analytics", icon: TrendingUp },
            ] as Array<{ id: AdvertiserTab; label: string; icon: typeof BarChart3 }>).map(({ id, label, icon: Icon }) => (
              <button
                key={id}
                onClick={() => setActiveTab(id)}
                className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  activeTab === id
                    ? "bg-white text-indigo-600 shadow-sm"
                    : "text-gray-600 hover:text-gray-900"
                }`}
              >
                <Icon className="w-4 h-4" />
                {label}
              </button>
            ))}
          </div>

          {/* Tab Content */}
          {activeTab === "campaigns" && (
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <h2 className="text-lg font-semibold text-gray-900 mb-4">
                Active Campaigns
              </h2>

              {isCampaignsLoading ? (
                <div className="animate-pulse flex space-x-4 p-4 border rounded-lg bg-gray-50">
                  <div className="flex-1 space-y-4 py-1">
                    <div className="h-4 bg-gray-200 rounded w-3/4"></div>
                    <div className="h-4 bg-gray-200 rounded w-1/2"></div>
                  </div>
                </div>
              ) : campaignViews.length > 0 ? (
                <div className="space-y-4 bg-gray-900 rounded-xl p-4">
                  <CampaignStats campaigns={campaignViews} />
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {campaignViews.map((campaign) => (
                      <CampaignCard key={campaign.campaign_id.toString()} campaign={campaign} />
                    ))}
                  </div>
                </div>
              ) : (
                <div className="text-center py-12 text-gray-500">
                  <BarChart3 className="w-12 h-12 mx-auto mb-3 opacity-30" />
                  <p>
                    No campaigns yet. Create your first campaign to get started.
                  </p>
                  <button
                    onClick={() => setActiveTab("create")}
                    className="mt-4 px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors text-sm"
                  >
                    Create Campaign
                  </button>
                </div>
              )}
            </div>
          )}

          {activeTab === "create" && (
            <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 max-w-2xl">
              <h2 className="text-lg font-semibold text-white mb-6">
                Create New Campaign
              </h2>

              {createdCampaignId !== null && (
                <div className="mb-4 p-3 bg-green-900/30 border border-green-700 rounded-lg text-green-300 text-sm">
                  Campaign #{createdCampaignId} created successfully on Stellar!
                </div>
              )}

              <CampaignForm
                onSuccess={(campaignId) => setCreatedCampaignId(campaignId)}
                onCancel={() => setActiveTab("campaigns")}
              />
            </div>
          )}

          {activeTab === "targeting" && (
            <div className="bg-gray-900 rounded-xl border border-gray-800 p-6 max-w-2xl">
              <h2 className="text-lg font-semibold text-white mb-1">
                Audience Targeting
              </h2>
              <p className="text-sm text-gray-400 mb-6">
                Configure geographic, interest, device, and quality targeting rules for your campaigns.
              </p>
              <TargetingForm
                initial={targetingConfig}
                onSave={async (config) => {
                  setTargetingConfig(config);
                }}
              />
            </div>
          )}

          {activeTab === "auctions" && (
            <div className="bg-gray-900 rounded-xl border border-gray-800 p-6">
              <h2 className="text-lg font-semibold text-white mb-1">
                Real-Time Bidding Marketplace
              </h2>
              <p className="text-sm text-gray-400 mb-4">
                Bid on open impression slots from publishers across the network.
              </p>
              <AuctionList
                auctions={(openAuctions ?? []) as unknown as Auction[]}
                isLoading={isAuctionsLoading}
                onRefresh={() => refetchAuctions()}
              />
            </div>
          )}

          {activeTab === "analytics" && (
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <h2 className="text-lg font-semibold text-gray-900 mb-4">
                Campaign Analytics
              </h2>
              {campaignViews.length > 0 ? (
                <div className="bg-gray-900 rounded-xl p-4">
                  <AnalyticsDashboard
                    campaigns={campaignViews}
                    timeframe={timeframe}
                    onTimeframeChange={setTimeframe}
                  />
                </div>
              ) : (
                <div className="text-center py-12 text-gray-500">
                  <TrendingUp className="w-12 h-12 mx-auto mb-3 opacity-30" />
                  <p>Analytics data will appear once campaigns are running.</p>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </ErrorBoundary>
  );
}
