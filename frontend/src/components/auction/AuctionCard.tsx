'use client';

import { Auction } from '@/types/contracts';
import { formatXlm, formatAddress } from '@/lib/display-utils';
import { clsx } from 'clsx';

interface AuctionCardProps {
  auction: Auction;
  onBid?: (auction: Auction) => void;
}

function AuctionStatus({ status, endTime }: { status: string; endTime: number | bigint }) {
  const now = Math.floor(Date.now() / 1000);
  const timeLeft = Number(endTime) - now;
  const isExpiringSoon = timeLeft > 0 && timeLeft < 300; // < 5 min

  const colors: Record<string, string> = {
    Open: 'bg-green-900/40 text-green-300 border border-green-700',
    Settled: 'bg-blue-900/40 text-blue-300 border border-blue-700',
    Cancelled: 'bg-red-900/40 text-red-300 border border-red-700',
  };

  return (
    <div className="flex items-center gap-2">
      <span
        className={clsx(
          'text-xs font-medium px-2 py-0.5 rounded-full',
          colors[status] || 'bg-gray-700 text-gray-300'
        )}
      >
        {status}
      </span>
      {isExpiringSoon && (
        <span className="text-xs text-orange-400 animate-pulse font-medium">Ending soon!</span>
      )}
    </div>
  );
}

function Countdown({ endTime }: { endTime: number | bigint }) {
  const now = Math.floor(Date.now() / 1000);
  const diff = Number(endTime) - now;

  if (diff <= 0) return <span className="text-gray-500 text-xs">Ended</span>;

  const h = Math.floor(diff / 3600);
  const m = Math.floor((diff % 3600) / 60);
  const s = diff % 60;

  const parts = [];
  if (h > 0) parts.push(`${h}h`);
  if (m > 0) parts.push(`${m}m`);
  parts.push(`${s}s`);

  return <span className="text-cyan-400 text-xs font-mono">{parts.join(' ')}</span>;
}

export function AuctionCard({ auction, onBid }: AuctionCardProps) {
  const isOpen = auction.status === 'Open';
  const floorXlm = formatXlm(BigInt(auction.floor_price));
  const winningBidXlm = auction.winning_bid ? formatXlm(BigInt(auction.winning_bid)) : null;

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-xl p-4 hover:border-gray-600 transition-all">
      <div className="flex items-start justify-between mb-3">
        <div>
          <p className="text-xs text-gray-500 font-mono">Auction #{auction.auction_id}</p>
          <p className="text-white font-medium text-sm mt-0.5 truncate max-w-[180px]">
            {auction.impression_slot}
          </p>
        </div>
        <AuctionStatus status={auction.status} endTime={auction.end_time} />
      </div>

      <div className="space-y-2 text-sm">
        <div className="flex justify-between">
          <span className="text-gray-400">Publisher</span>
          <span className="text-gray-200 font-mono text-xs">{formatAddress(auction.publisher)}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-400">Floor price</span>
          <span className="text-gray-200">{floorXlm} XLM</span>
        </div>
        {winningBidXlm && (
          <div className="flex justify-between">
            <span className="text-gray-400">Current bid</span>
            <span className="text-green-400 font-semibold">{winningBidXlm} XLM</span>
          </div>
        )}
        <div className="flex justify-between">
          <span className="text-gray-400">Bids</span>
          <span className="text-gray-200">{auction.bid_count}</span>
        </div>
        {isOpen && (
          <div className="flex justify-between">
            <span className="text-gray-400">Time left</span>
            <Countdown endTime={auction.end_time} />
          </div>
        )}
      </div>

      {isOpen && onBid && (
        <button
          onClick={() => onBid(auction)}
          className="w-full mt-4 bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2 rounded-lg transition-colors text-sm"
        >
          Place Bid
        </button>
      )}

      {auction.status === 'Settled' && auction.winner && (
        <div className="mt-3 pt-3 border-t border-gray-700">
          <p className="text-xs text-gray-500">
            Won by{' '}
            <span className="text-cyan-400 font-mono">{formatAddress(auction.winner)}</span>
          </p>
        </div>
      )}
    </div>
  );
}
