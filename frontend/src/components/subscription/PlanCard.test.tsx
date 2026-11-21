import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { PlanCard } from './PlanCard';
import type { SubscriptionPlan } from '@/types/contracts';

const plan: SubscriptionPlan & { features: string[] } = {
    tier: 'Growth',
    name: 'Growth',
    price_monthly: BigInt(2_990_000_000),
    price_annual: BigInt(29_900_000_000),
    max_campaigns: 0,
    max_impressions_per_month: BigInt(0),
    max_publishers: 0,
    analytics_enabled: true,
    api_access: false,
    features: ['Up to 25 campaigns', 'Full analytics'],
};

describe('PlanCard', () => {
    it('shows the price and features and subscribes on click', () => {
        const onSubscribe = vi.fn();
        render(<PlanCard plan={plan} onSubscribe={onSubscribe} />);

        expect(screen.getByText('299')).toBeInTheDocument();
        expect(screen.getByText('Up to 25 campaigns')).toBeInTheDocument();
        fireEvent.click(screen.getByRole('button', { name: 'Subscribe (Monthly)' }));

        expect(onSubscribe).toHaveBeenCalledWith(plan, false);
    });

    it('cannot subscribe again to the current plan', () => {
        render(<PlanCard plan={plan} isCurrentPlan onSubscribe={vi.fn()} />);

        expect(screen.getByRole('button', { name: 'Current Plan' })).toBeDisabled();
    });
});
