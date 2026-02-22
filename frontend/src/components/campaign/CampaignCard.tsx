'use client';

import { Campaign, CampaignStatus } from '@/types/contracts';
import { formatXlm, formatNumber, formatAddress } from '@/lib/display-utils';
import { clsx } from 'clsx';

interface CampaignCardProps {
  campaign: Campaign;
  onSelect?: (campaign: Campaign) => void;
}

function StatusBadge({ status }: { status: CampaignStatus }) {
  const colors: Record<string, string> = {
    Active: 'bg-green-900/40 text-green-300 border border-green-700',
    Paused: 'bg-yellow-900/40 text-yellow-300 border border-yellow-700',
    Completed: 'bg-blue-900/40 text-blue-300 border border-blue-700',
    Cancelled: 'bg-red-900/40 text-red-300 border border-red-700',
    Expired: 'bg-gray-900/40 text-gray-400 border border-gray-600',
  };
  return (
    <span className={clsx('text-xs font-medium px-2 py-0.5 rounded-full', colors[status])}>
      {status}
    </span>
  );
}

function ProgressBar({ spent, total }: { spent: bigint; total: bigint }) {
  const pct = total > BigInt(0) ? Number((spent * BigInt(100)) / total) : 0;
  return (
    <div className="w-full bg-gray-700 rounded-full h-1.5">
      <div
        className="bg-indigo-500 h-1.5 rounded-full transition-all"
        style={{ width: `${Math.min(pct, 100)}%` }}
      />
    </div>
  );
}

export function CampaignCard({ campaign, onSelect }: CampaignCardProps) {
  const budgetXlm = formatXlm(campaign.budget);
  const spentXlm = formatXlm(campaign.spent);
  const spentPct =
    campaign.budget > BigInt(0)
      ? Math.round(Number((campaign.spent * BigInt(100)) / campaign.budget))
      : 0;

  return (
    <div
      className={clsx(
        'bg-gray-800 border border-gray-700 rounded-xl p-4 transition-all',
        onSelect && 'cursor-pointer hover:border-indigo-500 hover:bg-gray-750'
      )}
      onClick={() => onSelect?.(campaign)}
    >
      <div className="flex items-start justify-between mb-3">
        <div className="flex-1 min-w-0">
          <h3 className="text-white font-semibold truncate">{campaign.title}</h3>
          <p className="text-gray-400 text-xs mt-0.5">
            by {formatAddress(campaign.advertiser)}
          </p>
        </div>
        <StatusBadge status={campaign.status} />
      </div>

      {/* Budget progress */}
      <div className="mb-3">
        <div className="flex justify-between text-xs text-gray-400 mb-1">
          <span>Budget spent</span>
          <span>{spentPct}%</span>
        </div>
        <ProgressBar spent={campaign.spent} total={campaign.budget} />
        <div className="flex justify-between text-xs mt-1">
          <span className="text-gray-400">{spentXlm} XLM used</span>
          <span className="text-gray-300">{budgetXlm} XLM total</span>
        </div>
      </div>

      {/* Stats row */}
      <div className="grid grid-cols-2 gap-3 mt-3 pt-3 border-t border-gray-700">
        <div>
          <p className="text-xs text-gray-500">Impressions</p>
          <p className="text-white font-medium text-sm">{formatNumber(campaign.impressions)}</p>
        </div>
        <div>
          <p className="text-xs text-gray-500">Clicks</p>
          <p className="text-white font-medium text-sm">{formatNumber(campaign.clicks)}</p>
        </div>
      </div>

      {campaign.expires_at && (
        <p className="text-xs text-gray-500 mt-2">
          Expires: {new Date(Number(campaign.expires_at) * 1000).toLocaleDateString()}
        </p>
      )}
    </div>
  );
}
