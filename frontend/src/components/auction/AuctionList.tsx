'use client';

import { useState } from 'react';
import { Auction } from '@/types/contracts';
import { AuctionCard } from './AuctionCard';
import { BidForm } from './BidForm';

interface AuctionListProps {
  auctions: Auction[];
  isLoading?: boolean;
  onRefresh?: () => void;
}

type FilterStatus = 'All' | 'Open' | 'Settled' | 'Cancelled';

export function AuctionList({ auctions, isLoading, onRefresh }: AuctionListProps) {
  const [filter, setFilter] = useState<FilterStatus>('All');
  const [biddingOn, setBiddingOn] = useState<Auction | null>(null);

  const filtered =
    filter === 'All' ? auctions : auctions.filter((a) => a.status === filter);

  if (biddingOn) {
    return (
      <BidForm
        auction={biddingOn}
        onSuccess={() => {
          setBiddingOn(null);
          onRefresh?.();
        }}
        onCancel={() => setBiddingOn(null)}
      />
    );
  }

  return (
    <div>
      {/* Filter tabs */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex gap-1 bg-gray-800 border border-gray-700 rounded-lg p-1">
          {(['All', 'Open', 'Settled', 'Cancelled'] as FilterStatus[]).map((f) => (
            <button
              key={f}
              onClick={() => setFilter(f)}
              className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${filter === f
                  ? 'bg-indigo-600 text-white'
                  : 'text-gray-400 hover:text-gray-200'
                }`}
            >
              {f}
              {f !== 'All' && (
                <span className="ml-1.5 text-xs opacity-70">
                  ({auctions.filter((a) => a.status === f).length})
                </span>
              )}
            </button>
          ))}
        </div>
        {onRefresh && (
          <button
            onClick={onRefresh}
            disabled={isLoading}
            className="text-xs text-gray-400 hover:text-gray-200 disabled:opacity-50 transition-colors"
          >
            {isLoading ? 'Loading...' : 'Refresh'}
          </button>
        )}
      </div>

      {isLoading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {[...Array(6)].map((_, i) => (
            <div key={i} className="bg-gray-800 border border-gray-700 rounded-xl h-52 animate-pulse" />
          ))}
        </div>
      ) : filtered.length === 0 ? (
        <div className="text-center py-12 text-gray-500">
          <p className="text-lg font-medium">No auctions found</p>
          <p className="text-sm mt-1">
            {filter === 'Open' ? 'No open auctions right now.' : `No ${filter.toLowerCase()} auctions.`}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {filtered.map((auction) => (
            <AuctionCard
              key={auction.auction_id}
              auction={auction}
              onBid={auction.status === 'Open' ? setBiddingOn : undefined}
            />
          ))}
        </div>
      )}
    </div>
  );
}
