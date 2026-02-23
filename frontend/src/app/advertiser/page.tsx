'use client';

import { useState } from 'react';
import { PlusCircle, BarChart3, Settings, DollarSign, TrendingUp, Eye, MousePointer } from 'lucide-react';
import { useWalletStore } from '@/store/wallet-store';
import { WalletConnectButton } from '@/components/wallet/WalletModal';
import { useCreateCampaign } from '@/hooks/useContract';
import { formatXlm, formatNumber } from '@/lib/display-utils';

interface CampaignForm {
  title: string;
  budgetXlm: string;
  dailyBudgetXlm: string;
  durationDays: string;
  contentId: string;
}

const EMPTY_FORM: CampaignForm = {
  title: '',
  budgetXlm: '',
  dailyBudgetXlm: '',
  durationDays: '30',
  contentId: '',
};

export default function AdvertiserPage() {
  const { address, isConnected } = useWalletStore();
  const { createCampaign, isPending, isSuccess, isError } = useCreateCampaign();
  const [form, setForm] = useState<CampaignForm>(EMPTY_FORM);
  const [activeTab, setActiveTab] = useState<'campaigns' | 'create' | 'analytics'>('campaigns');

  if (!isConnected) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center max-w-md p-8 bg-white rounded-2xl shadow-sm border border-gray-200">
          <div className="w-16 h-16 bg-indigo-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <DollarSign className="w-8 h-8 text-indigo-600" />
          </div>
          <h2 className="text-2xl font-bold text-gray-900 mb-2">Advertiser Dashboard</h2>
          <p className="text-gray-600 mb-6">
            Connect your Freighter wallet to manage campaigns on the Stellar network.
          </p>
          <WalletConnectButton />
        </div>
      </div>
    );
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createCampaign({
      title: form.title,
      budgetXlm: parseFloat(form.budgetXlm) || 0,
      dailyBudgetXlm: parseFloat(form.dailyBudgetXlm) || 0,
      durationDays: parseInt(form.durationDays) || 30,
      contentId: form.contentId,
    });
  };

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Page Header */}
      <div className="bg-white border-b border-gray-200 px-4 sm:px-6 lg:px-8 py-6">
        <div className="max-w-7xl mx-auto flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">Advertiser Dashboard</h1>
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
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
          {[
            { icon: BarChart3, label: 'Active Campaigns', value: '0', color: 'blue' },
            { icon: Eye, label: 'Total Impressions', value: '0', color: 'green' },
            { icon: MousePointer, label: 'Total Clicks', value: '0', color: 'purple' },
            { icon: TrendingUp, label: 'Total Spent', value: '0 XLM', color: 'orange' },
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
        <div className="flex gap-1 mb-6 bg-gray-100 p-1 rounded-lg w-fit">
          {[
            { id: 'campaigns', label: 'My Campaigns', icon: BarChart3 },
            { id: 'create', label: 'Create Campaign', icon: PlusCircle },
            { id: 'analytics', label: 'Analytics', icon: TrendingUp },
          ].map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setActiveTab(id as any)}
              className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors ${activeTab === id
                  ? 'bg-white text-indigo-600 shadow-sm'
                  : 'text-gray-600 hover:text-gray-900'
                }`}
            >
              <Icon className="w-4 h-4" />
              {label}
            </button>
          ))}
        </div>

        {/* Tab Content */}
        {activeTab === 'campaigns' && (
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-4">Active Campaigns</h2>
            <div className="text-center py-12 text-gray-500">
              <BarChart3 className="w-12 h-12 mx-auto mb-3 opacity-30" />
              <p>No campaigns yet. Create your first campaign to get started.</p>
              <button
                onClick={() => setActiveTab('create')}
                className="mt-4 px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors text-sm"
              >
                Create Campaign
              </button>
            </div>
          </div>
        )}

        {activeTab === 'create' && (
          <div className="bg-white rounded-xl border border-gray-200 p-6 max-w-2xl">
            <h2 className="text-lg font-semibold text-gray-900 mb-6">Create New Campaign</h2>

            {isSuccess && (
              <div className="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg text-green-700 text-sm">
                Campaign created successfully on Stellar!
              </div>
            )}
            {isError && (
              <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
                Error creating campaign. Check your XLM balance and try again.
              </div>
            )}

            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Campaign Title</label>
                <input
                  type="text"
                  value={form.title}
                  onChange={(e) => setForm({ ...form, title: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                  placeholder="My Awesome Campaign"
                  required
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Content ID (IPFS Hash)</label>
                <input
                  type="text"
                  value={form.contentId}
                  onChange={(e) => setForm({ ...form, contentId: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent font-mono text-sm"
                  placeholder="QmXxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
                  required
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Total Budget (XLM)</label>
                  <input
                    type="number"
                    value={form.budgetXlm}
                    onChange={(e) => setForm({ ...form, budgetXlm: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                    placeholder="1000"
                    min="1"
                    step="0.01"
                    required
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Daily Budget (XLM)</label>
                  <input
                    type="number"
                    value={form.dailyBudgetXlm}
                    onChange={(e) => setForm({ ...form, dailyBudgetXlm: e.target.value })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                    placeholder="50"
                    min="0.01"
                    step="0.01"
                    required
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Duration (days)</label>
                <input
                  type="number"
                  value={form.durationDays}
                  onChange={(e) => setForm({ ...form, durationDays: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                  min="1"
                  max="365"
                  required
                />
              </div>

              <button
                type="submit"
                disabled={isPending}
                className="w-full py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isPending ? 'Creating on Stellar...' : 'Create Campaign'}
              </button>

              <p className="text-xs text-gray-500 text-center">
                This will submit a Soroban transaction. You will be prompted to sign with Freighter.
              </p>
            </form>
          </div>
        )}

        {activeTab === 'analytics' && (
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-4">Campaign Analytics</h2>
            <div className="text-center py-12 text-gray-500">
              <TrendingUp className="w-12 h-12 mx-auto mb-3 opacity-30" />
              <p>Analytics data will appear once campaigns are running.</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
