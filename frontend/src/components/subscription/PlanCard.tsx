'use client';

import { SubscriptionPlan } from '@/types/contracts';
import { clsx } from 'clsx';

interface PlanCardProps {
  plan: SubscriptionPlan;
  isCurrentPlan?: boolean;
  isAnnual?: boolean;
  onSubscribe?: (plan: SubscriptionPlan, annual: boolean) => void;
  isLoading?: boolean;
}

const TIER_GRADIENT: Record<string, string> = {
  Starter: 'from-gray-700 to-gray-800',
  Growth: 'from-indigo-900 to-gray-800',
  Business: 'from-purple-900 to-gray-800',
  Enterprise: 'from-cyan-900 to-gray-800',
};

const TIER_ACCENT: Record<string, string> = {
  Starter: 'border-gray-600',
  Growth: 'border-indigo-600',
  Business: 'border-purple-600',
  Enterprise: 'border-cyan-600',
};

const TIER_BUTTON: Record<string, string> = {
  Starter: 'bg-gray-600 hover:bg-gray-500',
  Growth: 'bg-indigo-600 hover:bg-indigo-500',
  Business: 'bg-purple-600 hover:bg-purple-500',
  Enterprise: 'bg-cyan-700 hover:bg-cyan-600',
};

export function PlanCard({ plan, isCurrentPlan, isAnnual = false, onSubscribe, isLoading }: PlanCardProps) {
  const priceXlm = (Number(isAnnual ? plan.price_annual : plan.price_monthly) / 1e7).toFixed(0);
  const annualSavings = isAnnual
    ? Math.round(
      (1 - Number(plan.price_annual) / (Number(plan.price_monthly) * 12)) * 100
    )
    : 0;

  return (
    <div
      className={clsx(
        'relative bg-gradient-to-b rounded-xl border-2 p-5 transition-all',
        TIER_GRADIENT[plan.tier],
        TIER_ACCENT[plan.tier],
        isCurrentPlan && 'ring-2 ring-white/20'
      )}
    >
      {isCurrentPlan && (
        <div className="absolute -top-3 left-1/2 -translate-x-1/2 bg-white text-gray-900 text-xs font-bold px-3 py-0.5 rounded-full">
          Current Plan
        </div>
      )}

      {isAnnual && annualSavings > 0 && (
        <div className="absolute -top-3 right-4 bg-green-600 text-white text-xs font-bold px-2 py-0.5 rounded-full">
          Save {annualSavings}%
        </div>
      )}

      <h3 className="text-white font-bold text-lg mb-1">{plan.tier}</h3>

      <div className="mb-4">
        <span className="text-3xl font-extrabold text-white">{priceXlm}</span>
        <span className="text-gray-400 text-sm ml-1">XLM / {isAnnual ? 'year' : 'month'}</span>
      </div>

      {(plan as any).features && (
        <ul className="space-y-2 mb-5">
          {(plan as any).features.map((feature: string) => (
            <li key={feature} className="flex items-start gap-2 text-sm text-gray-300">
              <span className="text-green-400 flex-shrink-0 mt-0.5">✓</span>
              {feature}
            </li>
          ))}
        </ul>
      )}

      <button
        onClick={() => onSubscribe?.(plan, isAnnual)}
        disabled={isCurrentPlan || isLoading}
        className={clsx(
          'w-full py-2 rounded-lg text-white font-medium text-sm transition-colors',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          TIER_BUTTON[plan.tier]
        )}
      >
        {isCurrentPlan ? 'Current Plan' : isLoading ? 'Processing...' : `Subscribe ${isAnnual ? '(Annual)' : '(Monthly)'}`}
      </button>
    </div>
  );
}
