'use client';

import { useState } from 'react';
import { Auction } from '@/types/contracts';
import { usePlaceBid } from '@/hooks/useContract';
import { formatXlm } from '@/lib/display-utils';
import { xlmToStroops } from '@/lib/stellar-config';

interface BidFormProps {
  auction: Auction;
  campaignId?: number;
  onSuccess?: () => void;
  onCancel?: () => void;
}

export function BidForm({ auction, campaignId, onSuccess, onCancel }: BidFormProps) {
  const [bidXlm, setBidXlm] = useState('');
  const [selectedCampaign, setSelectedCampaign] = useState(campaignId?.toString() ?? '');
  const [error, setError] = useState<string | null>(null);
  const { mutateAsync: placeBid, isPending } = usePlaceBid();

  const floorXlm = Number(formatXlm(BigInt(auction.floorPrice)));
  const currentBidXlm = auction.winningBid
    ? Number(formatXlm(BigInt(auction.winningBid)))
    : null;
  const minBid = currentBidXlm ? currentBidXlm * 1.05 : floorXlm; // 5% increment or floor

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    const amount = parseFloat(bidXlm);
    if (isNaN(amount) || amount <= 0) { setError('Enter a valid bid amount'); return; }
    if (amount < minBid) {
      setError(`Minimum bid is ${minBid.toFixed(4)} XLM`);
      return;
    }
    if (!selectedCampaign) { setError('Select a campaign'); return; }

    try {
      await placeBid({
        auctionId: auction.auctionId,
        campaignId: parseInt(selectedCampaign),
        amount: xlmToStroops(amount),
      });
      onSuccess?.();
    } catch (err: any) {
      setError(err?.message || 'Failed to place bid');
    }
  };

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-xl p-5">
      <h3 className="text-white font-semibold mb-1">Place Bid</h3>
      <p className="text-gray-400 text-xs mb-4">
        Auction #{auction.auctionId} &mdash; {auction.impressionSlot}
      </p>

      <div className="grid grid-cols-2 gap-3 mb-4 text-sm">
        <div className="bg-gray-700/50 rounded-lg p-3">
          <p className="text-gray-400 text-xs">Floor Price</p>
          <p className="text-white font-medium">{floorXlm} XLM</p>
        </div>
        <div className="bg-gray-700/50 rounded-lg p-3">
          <p className="text-gray-400 text-xs">
            {currentBidXlm ? 'Current Bid' : 'No bids yet'}
          </p>
          <p className="text-green-400 font-medium">
            {currentBidXlm ? `${currentBidXlm.toFixed(4)} XLM` : '—'}
          </p>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="space-y-3">
        <div>
          <label htmlFor="bid-campaign-id" className="block text-sm font-medium text-gray-300 mb-1">
            Campaign ID
          </label>
          <input
            id="bid-campaign-id"
            type="number"
            value={selectedCampaign}
            onChange={(e) => setSelectedCampaign(e.target.value)}
            placeholder="Enter campaign ID"
            className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
          />
        </div>

        <div>
          <label htmlFor="bid-amount" className="block text-sm font-medium text-gray-300 mb-1">
            Bid Amount (XLM)
          </label>
          <div className="relative">
            <input
              id="bid-amount"
              type="number"
              value={bidXlm}
              onChange={(e) => setBidXlm(e.target.value)}
              placeholder={minBid.toFixed(4)}
              min={minBid}
              step="0.0001"
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 pr-12 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
            />
            <span className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 text-sm">
              XLM
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1">Minimum: {minBid.toFixed(4)} XLM</p>
        </div>

        {error && (
          <div className="bg-red-900/30 border border-red-700 rounded-lg px-3 py-2 text-red-300 text-xs">
            {error}
          </div>
        )}

        <div className="flex gap-3">
          <button
            type="submit"
            disabled={isPending}
            className="flex-1 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium py-2 px-4 rounded-lg transition-colors text-sm"
          >
            {isPending ? 'Submitting...' : 'Submit Bid'}
          </button>
          {onCancel && (
            <button
              type="button"
              onClick={onCancel}
              className="px-4 py-2 border border-gray-600 text-gray-300 rounded-lg hover:bg-gray-700 transition-colors text-sm"
            >
              Cancel
            </button>
          )}
        </div>
      </form>
    </div>
  );
}
