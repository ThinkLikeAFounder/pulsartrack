'use client';

import { Publisher } from '@/types/contracts';
import { formatXlm, formatNumber, formatAddress } from '@/lib/display-utils';
import { ReputationBadge } from './ReputationBadge';
import { clsx } from 'clsx';

interface PublisherCardProps {
  publisher: Publisher;
  rank?: number;
  onSelect?: (publisher: Publisher) => void;
}

function StatusDot({ status }: { status: string }) {
  return (
    <span className="flex items-center gap-1.5">
      <span
        className={clsx(
          'w-2 h-2 rounded-full',
          status === 'Active' ? 'bg-green-400' : 'bg-gray-500'
        )}
      />
      <span className="text-xs text-gray-400">{status}</span>
    </span>
  );
}

export function PublisherCard({ publisher, rank, onSelect }: PublisherCardProps) {
  return (
    <div
      className={clsx(
        'bg-gray-800 border border-gray-700 rounded-xl p-4 transition-all',
        onSelect && 'cursor-pointer hover:border-indigo-500 hover:bg-gray-750'
      )}
      onClick={() => onSelect?.(publisher)}
    >
      <div className="flex items-start gap-3">
        {rank !== undefined && (
          <div className="flex-shrink-0 w-8 h-8 bg-gray-700 rounded-lg flex items-center justify-center">
            <span className="text-sm font-bold text-gray-300">#{rank}</span>
          </div>
        )}

        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between">
            <h3 className="text-white font-semibold truncate">{publisher.display_name}</h3>
            <StatusDot status={publisher.status} />
          </div>
          {publisher.website && (
            <a
              href={publisher.website}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-indigo-400 hover:text-indigo-300 truncate block"
              onClick={(e) => e.stopPropagation()}
            >
              {publisher.website}
            </a>
          )}
          <p className="text-xs text-gray-500 font-mono mt-0.5">
            {formatAddress(publisher.publisher)}
          </p>
        </div>
      </div>

      <div className="mt-3">
        <ReputationBadge score={(publisher as any).reputation_score || 0} size="sm" />
      </div>

      <div className="grid grid-cols-2 gap-3 mt-3 pt-3 border-t border-gray-700">
        <div>
          <p className="text-xs text-gray-500">Impressions</p>
          <p className="text-white font-medium text-sm">
            {formatNumber(publisher.impressions_served)}
          </p>
        </div>
        <div>
          <p className="text-xs text-gray-500">Earnings</p>
          <p className="text-green-400 font-medium text-sm">
            {formatXlm(publisher.earnings_total)} XLM
          </p>
        </div>
      </div>
    </div>
  );
}
