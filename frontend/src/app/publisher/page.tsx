'use client';

import { useState } from 'react';
import { Radio, DollarSign, Star, Shield, TrendingUp, Clock, CheckCircle } from 'lucide-react';
import { useWalletStore } from '@/store/wallet-store';
import { WalletConnectButton } from '@/components/wallet/WalletModal';
import { usePublisherReputation } from '@/hooks/useContract';
import { formatAddress, formatScore } from '@/lib/display-utils';

const SUBSCRIPTION_PLANS = [
  {
    name: 'Starter',
    priceMonthly: '99 XLM',
    priceAnnual: '990 XLM',
    features: ['Up to 5 campaigns', '100K impressions/mo', '10 publishers', 'Basic analytics'],
    color: 'gray',
  },
  {
    name: 'Growth',
    priceMonthly: '299 XLM',
    priceAnnual: '2,990 XLM',
    features: ['Up to 25 campaigns', '500K impressions/mo', '50 publishers', 'Full analytics'],
    color: 'blue',
    popular: true,
  },
  {
    name: 'Business',
    priceMonthly: '799 XLM',
    priceAnnual: '7,990 XLM',
    features: ['Up to 100 campaigns', '2M impressions/mo', '200 publishers', 'Analytics + API'],
    color: 'purple',
  },
  {
    name: 'Enterprise',
    priceMonthly: '1,999 XLM',
    priceAnnual: '19,990 XLM',
    features: ['Unlimited campaigns', '10M impressions/mo', '1000 publishers', 'Analytics + API'],
    color: 'indigo',
  },
];

import { ErrorBoundary } from '@/components/ErrorBoundary';

export default function PublisherPage() {
  const { address, isConnected } = useWalletStore();
  const { data: reputation } = usePublisherReputation(address || '');
  const [activeTab, setActiveTab] = useState<'overview' | 'auctions' | 'earnings' | 'subscription'>('overview');

  if (!isConnected) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center max-w-md p-8 bg-white rounded-2xl shadow-sm border border-gray-200">
          <div className="w-16 h-16 bg-indigo-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <Radio className="w-8 h-8 text-indigo-600" />
          </div>
          <h2 className="text-2xl font-bold text-gray-900 mb-2">Publisher Dashboard</h2>
          <p className="text-gray-600 mb-6">
            Connect your Freighter wallet to earn XLM by serving ads on the Stellar network.
          </p>
          <WalletConnectButton />
        </div>
      </div>
    );
  }

  const reputationScore = reputation ? (reputation as any).score ?? 500 : 500;

  return (
    <ErrorBoundary name="PublisherPage" resetKeys={[activeTab]}>
      <div className="min-h-screen bg-gray-50">
        {/* Page Header */}
        <div className="bg-white border-b border-gray-200 px-4 sm:px-6 lg:px-8 py-6">
          <div className="max-w-7xl mx-auto flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">Publisher Dashboard</h1>
              <p className="text-sm text-gray-500 mt-1 font-mono">{formatAddress(address || '')}</p>
            </div>
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-1 px-3 py-1 bg-amber-100 text-amber-700 rounded-full text-sm font-medium">
                <Star className="w-3.5 h-3.5" />
                Rep: {reputationScore}/1000
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
              { icon: DollarSign, label: 'Total Earned', value: '0 XLM', color: 'green' },
              { icon: TrendingUp, label: 'Impressions Served', value: '0', color: 'blue' },
              { icon: Star, label: 'Reputation Score', value: `${reputationScore}`, color: 'amber' },
              { icon: Clock, label: 'Active Auctions', value: '0', color: 'purple' },
            ].map(({ icon: Icon, label, value, color }) => (
              <div key={label} className="bg-white p-4 rounded-xl border border-gray-200">
                <div className="flex items-center gap-3">
                  <div className={`w-10 h-10 bg-${color}-100 rounded-lg flex items-center justify-center`}>
                    <Icon className={`w-5 h-5 text-${color}-600`} />
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
            {[
              { id: 'overview', label: 'Overview' },
              { id: 'auctions', label: 'RTB Auctions' },
              { id: 'earnings', label: 'Earnings' },
              { id: 'subscription', label: 'Subscription Plans' },
            ].map(({ id, label }) => (
              <button
                key={id}
                onClick={() => setActiveTab(id as any)}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${activeTab === id
                    ? 'bg-white text-indigo-600 shadow-sm'
                    : 'text-gray-600 hover:text-gray-900'
                  }`}
              >
                {label}
              </button>
            ))}
          </div>

          {activeTab === 'overview' && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="bg-white rounded-xl border border-gray-200 p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Verification Status</h3>
                <div className="space-y-3">
                  {[
                    { label: 'Wallet Connected', done: true },
                    { label: 'Publisher Registered', done: false },
                    { label: 'KYC Verified', done: false },
                    { label: 'Reputation Initialized', done: false },
                  ].map(({ label, done }) => (
                    <div key={label} className="flex items-center gap-3">
                      <CheckCircle className={`w-5 h-5 ${done ? 'text-green-500' : 'text-gray-300'}`} />
                      <span className={`text-sm ${done ? 'text-gray-900' : 'text-gray-500'}`}>{label}</span>
                    </div>
                  ))}
                </div>
              </div>

              <div className="bg-white rounded-xl border border-gray-200 p-6">
                <h3 className="text-lg font-semibold text-gray-900 mb-4">Publisher Tiers</h3>
                <div className="space-y-2">
                  {[
                    { tier: 'Bronze', min: 0, max: 399, color: 'amber' },
                    { tier: 'Silver', min: 400, max: 599, color: 'gray' },
                    { tier: 'Gold', min: 600, max: 799, color: 'yellow' },
                    { tier: 'Platinum', min: 800, max: 1000, color: 'blue' },
                  ].map(({ tier, min, max, color }) => (
                    <div
                      key={tier}
                      className={`flex items-center justify-between px-3 py-2 rounded-lg ${reputationScore >= min && reputationScore <= max
                          ? 'bg-indigo-50 border border-indigo-200'
                          : 'bg-gray-50'
                        }`}
                    >
                      <span className="text-sm font-medium text-gray-900">{tier}</span>
                      <span className="text-xs text-gray-500">{min} - {max} score</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {activeTab === 'auctions' && (
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <h2 className="text-lg font-semibold text-gray-900 mb-4">Real-Time Bidding Auctions</h2>
              <div className="text-center py-12 text-gray-500">
                <Clock className="w-12 h-12 mx-auto mb-3 opacity-30" />
                <p>No active auctions. Create an impression slot to start receiving bids.</p>
                <button className="mt-4 px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors text-sm">
                  Create Impression Slot
                </button>
              </div>
            </div>
          )}

          {activeTab === 'earnings' && (
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <h2 className="text-lg font-semibold text-gray-900 mb-4">XLM Earnings</h2>
              <div className="text-center py-12 text-gray-500">
                <DollarSign className="w-12 h-12 mx-auto mb-3 opacity-30" />
                <p>No earnings yet. Start serving ads to earn XLM.</p>
              </div>
            </div>
          )}

          {activeTab === 'subscription' && (
            <div>
              <h2 className="text-lg font-semibold text-gray-900 mb-6">Subscription Plans</h2>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                {SUBSCRIPTION_PLANS.map((plan) => (
                  <div
                    key={plan.name}
                    className={`bg-white rounded-xl border-2 p-6 relative ${plan.popular ? 'border-indigo-500' : 'border-gray-200'
                      }`}
                  >
                    {plan.popular && (
                      <div className="absolute -top-3 left-1/2 -translate-x-1/2 px-3 py-1 bg-indigo-600 text-white text-xs rounded-full">
                        Most Popular
                      </div>
                    )}
                    <h3 className="text-lg font-bold text-gray-900">{plan.name}</h3>
                    <p className="text-2xl font-bold text-indigo-600 mt-2">{plan.priceMonthly}</p>
                    <p className="text-sm text-gray-500">per month</p>
                    <p className="text-xs text-gray-500 mt-1">{plan.priceAnnual} / year (save 17%)</p>
                    <ul className="mt-4 space-y-2">
                      {plan.features.map((feature) => (
                        <li key={feature} className="flex items-center gap-2 text-sm text-gray-600">
                          <CheckCircle className="w-4 h-4 text-green-500 shrink-0" />
                          {feature}
                        </li>
                      ))}
                    </ul>
                    <button className="w-full mt-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors text-sm font-medium">
                      Subscribe with XLM
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </ErrorBoundary>
  );
}
