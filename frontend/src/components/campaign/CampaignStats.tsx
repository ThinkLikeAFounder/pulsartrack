'use client';

import { Campaign } from '@/types/contracts';
import { formatXlm, formatNumber } from '@/lib/display-utils';

interface CampaignStatsProps {
  campaigns: Campaign[];
}

export function CampaignStats({ campaigns }: CampaignStatsProps) {
  const active = campaigns.filter((c) => c.status === 'Active').length;
  const totalBudget = campaigns.reduce((acc, c) => acc + c.budget, BigInt(0));
  const totalSpent = campaigns.reduce((acc, c) => acc + c.spent, BigInt(0));
  const totalImpressions = campaigns.reduce((acc, c) => acc + BigInt(c.impressions), BigInt(0));
  const totalClicks = campaigns.reduce((acc, c) => acc + BigInt(c.clicks), BigInt(0));
  const ctr =
    totalImpressions > BigInt(0)
      ? ((Number(totalClicks) / Number(totalImpressions)) * 100).toFixed(2)
      : '0.00';

  const stats = [
    { label: 'Active Campaigns', value: String(active), sub: `of ${campaigns.length} total` },
    { label: 'Total Budget', value: `${formatXlm(totalBudget)} XLM`, sub: 'allocated' },
    { label: 'Total Spent', value: `${formatXlm(totalSpent)} XLM`, sub: 'across all campaigns' },
    { label: 'Impressions', value: formatNumber(Number(totalImpressions)), sub: 'total served' },
    { label: 'Clicks', value: formatNumber(Number(totalClicks)), sub: `${ctr}% CTR` },
  ];

  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
      {stats.map(({ label, value, sub }) => (
        <div
          key={label}
          className="bg-gray-800 border border-gray-700 rounded-xl p-4 text-center"
        >
          <p className="text-2xl font-bold text-white">{value}</p>
          <p className="text-xs font-medium text-indigo-400 mt-0.5">{label}</p>
          <p className="text-xs text-gray-500 mt-0.5">{sub}</p>
        </div>
      ))}
    </div>
  );
}
