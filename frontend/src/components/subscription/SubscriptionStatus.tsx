'use client';

import { Subscription } from '@/types/contracts';
import { clsx } from 'clsx';

interface SubscriptionStatusProps {
  subscription: Subscription | null;
  onCancel?: () => void;
  onRenew?: () => void;
}

const TIER_COLORS: Record<string, string> = {
  Starter: 'text-gray-300 border-gray-600',
  Growth: 'text-indigo-300 border-indigo-600',
  Business: 'text-purple-300 border-purple-600',
  Enterprise: 'text-cyan-300 border-cyan-600',
};

export function SubscriptionStatus({ subscription, onCancel, onRenew }: SubscriptionStatusProps) {
  if (!subscription) {
    return (
      <div className="bg-gray-800 border border-gray-700 rounded-xl p-5 text-center">
        <p className="text-gray-400 text-sm">No active subscription</p>
        <p className="text-xs text-gray-500 mt-1">Subscribe to unlock premium features</p>
      </div>
    );
  }

  const expiresDate = new Date(Number(subscription.expires_at) * 1000);
  const now = Date.now();
  const daysLeft = Math.max(0, Math.floor((Number(subscription.expires_at) * 1000 - now) / 86400000));
  const isExpiringSoon = daysLeft <= 7 && daysLeft > 0;
  const isExpired = daysLeft === 0;
  const tierStyle = TIER_COLORS[subscription.tier] || TIER_COLORS.Starter;
  const pricePaid = (Number(subscription.amount_paid) / 1e7).toFixed(2);

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-xl p-5 space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-xs text-gray-500 uppercase tracking-wide">Active Plan</p>
          <span
            className={clsx(
              'text-xl font-bold border-b-2 pb-0.5',
              tierStyle
            )}
          >
            {subscription.tier}
          </span>
        </div>
        <div className="text-right">
          <p className="text-xs text-gray-500">Billing</p>
          <p className="text-sm text-gray-200 font-medium">
            {subscription.is_annual ? 'Annual' : 'Monthly'}
          </p>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-3 text-sm">
        <div className="bg-gray-700/50 rounded-lg p-3">
          <p className="text-xs text-gray-400">Expires</p>
          <p className={clsx(
            'font-medium',
            isExpired ? 'text-red-400' : isExpiringSoon ? 'text-yellow-400' : 'text-gray-200'
          )}>
            {isExpired
              ? 'Expired'
              : isExpiringSoon
                ? `${daysLeft} days left`
                : expiresDate.toLocaleDateString()}
          </p>
        </div>
        <div className="bg-gray-700/50 rounded-lg p-3">
          <p className="text-xs text-gray-400">Last paid</p>
          <p className="text-gray-200 font-medium">{pricePaid} XLM</p>
        </div>
      </div>

      <div className="flex items-center justify-between text-xs text-gray-500 pt-1">
        <span>Auto-renew: {subscription.auto_renew ? '✓ On' : '✗ Off'}</span>
      </div>

      {/* Actions */}
      <div className="flex gap-2 pt-1">
        {(isExpiringSoon || isExpired) && onRenew && (
          <button
            onClick={onRenew}
            className="flex-1 bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2 px-4 rounded-lg text-sm transition-colors"
          >
            Renew
          </button>
        )}
        {!isExpired && onCancel && (
          <button
            onClick={onCancel}
            className="px-4 py-2 border border-red-800/50 text-red-400 hover:bg-red-900/20 rounded-lg text-sm transition-colors"
          >
            Cancel
          </button>
        )}
      </div>
    </div>
  );
}
