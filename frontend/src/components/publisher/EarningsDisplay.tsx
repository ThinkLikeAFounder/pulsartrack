'use client';

import { formatXlm, formatNumber, formatBps } from '@/lib/display-utils';

interface EarningPeriod {
  label: string;
  amount: bigint;
  impressions: number;
}

/** Revenue split as configured on-chain, in basis points (10_000 = 100%). */
export interface RevenueSplit {
  publisherPct: number | bigint;
  treasuryPct: number | bigint;
  platformPct: number | bigint;
  burnPct: number | bigint;
}

interface EarningsDisplayProps {
  totalEarnings: bigint;
  periods?: EarningPeriod[];
  pendingPayout?: bigint;
  onWithdraw?: () => void;
  isWithdrawing?: boolean;
  /** Split read from the revenue-settlement contract; omit while unavailable. */
  revenueSplit?: RevenueSplit;
  isSplitLoading?: boolean;
}

export function EarningsDisplay({
  totalEarnings,
  periods = [],
  pendingPayout = BigInt(0),
  onWithdraw,
  isWithdrawing,
  revenueSplit,
  isSplitLoading,
}: EarningsDisplayProps) {
  const splitRows = revenueSplit
    ? [
        { label: 'Publisher (you)', bps: revenueSplit.publisherPct, color: 'bg-green-500' },
        { label: 'Treasury', bps: revenueSplit.treasuryPct, color: 'bg-blue-500' },
        { label: 'Platform fee', bps: revenueSplit.platformPct, color: 'bg-purple-500' },
        { label: 'PULSAR burn', bps: revenueSplit.burnPct, color: 'bg-red-500' },
      ]
    : [];

  const maxAmount = periods.reduce(
    (max, p) => (p.amount > max ? p.amount : max),
    BigInt(1)
  );

  return (
    <div className="space-y-4">
      {/* Summary */}
      <div className="grid grid-cols-2 gap-3">
        <div className="bg-gray-700/50 rounded-xl p-4">
          <p className="text-xs text-gray-400 uppercase tracking-wide">Total Earnings</p>
          <p className="text-2xl font-bold text-green-400 mt-1">
            {formatXlm(totalEarnings)}
          </p>
        </div>
        <div className="bg-gray-700/50 rounded-xl p-4">
          <p className="text-xs text-gray-400 uppercase tracking-wide">Pending Payout</p>
          <p className="text-2xl font-bold text-yellow-400 mt-1">
            {formatXlm(pendingPayout)}
          </p>
          {pendingPayout > BigInt(0) && onWithdraw && (
            <button
              onClick={onWithdraw}
              disabled={isWithdrawing}
              className="mt-2 text-xs bg-yellow-700/50 hover:bg-yellow-700 disabled:opacity-50 text-yellow-200 px-3 py-1 rounded-lg transition-colors"
            >
              {isWithdrawing ? 'Processing...' : 'Withdraw'}
            </button>
          )}
        </div>
      </div>

      {/* Period bars */}
      {periods.length > 0 && (
        <div className="bg-gray-800 border border-gray-700 rounded-xl p-4">
          <h4 className="text-sm font-medium text-gray-300 mb-3">Earnings History</h4>
          <div className="space-y-2">
            {periods.map((period) => {
              const pct = maxAmount > BigInt(0)
                ? Number((period.amount * BigInt(100)) / maxAmount)
                : 0;
              return (
                <div key={period.label}>
                  <div className="flex justify-between text-xs text-gray-400 mb-1">
                    <span>{period.label}</span>
                    <span className="text-gray-200">
                      {formatXlm(period.amount)}
                      <span className="text-gray-500 ml-1">
                        ({formatNumber(period.impressions)} imp.)
                      </span>
                    </span>
                  </div>
                  <div className="w-full bg-gray-700 rounded-full h-1.5">
                    <div
                      className="bg-green-500 h-1.5 rounded-full transition-all"
                      style={{ width: `${pct}%` }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Revenue split info */}
      <div className="bg-gray-800 border border-gray-700 rounded-xl p-4">
        <h4 className="text-sm font-medium text-gray-300 mb-2">Revenue Split</h4>
        {splitRows.length > 0 ? (
          <div className="space-y-1.5 text-xs">
            {splitRows.map(({ label, bps, color }) => (
              <div key={label} className="flex items-center gap-2">
                <span className={`w-2 h-2 rounded-full flex-shrink-0 ${color}`} />
                <span className="text-gray-400 flex-1">{label}</span>
                <span className="text-gray-200 font-medium">{formatBps(bps)}</span>
              </div>
            ))}
            <p className="text-gray-500 pt-1">
              Configured on-chain by the revenue-settlement contract; may change.
            </p>
          </div>
        ) : (
          <p className="text-xs text-gray-500">
            {isSplitLoading
              ? 'Loading revenue split...'
              : 'Revenue split unavailable — could not be read from the revenue-settlement contract.'}
          </p>
        )}
      </div>
    </div>
  );
}
