import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { AnalyticsDashboard } from './AnalyticsDashboard';
import { toCampaign } from '@/lib/contract-adapters';

vi.mock('@/hooks/useAnalytics', () => ({
    useAnalyticsTimeseries: () => ({ data: [], loading: false, error: 'offline' }),
}));

const campaigns = [
    toCampaign({ id: 1, budget: BigInt(1_000_000_000), remaining_budget: BigInt(600_000_000), current_views: BigInt(3_000), status: ['Active'] } as never),
    toCampaign({ id: 2, budget: BigInt(500_000_000), remaining_budget: BigInt(500_000_000), current_views: BigInt(0), status: ['Paused'] } as never),
];

describe('AnalyticsDashboard', () => {
    it('summarises the campaigns and lists each one', () => {
        render(<AnalyticsDashboard campaigns={campaigns} />);

        expect(screen.getByText('Campaign Breakdown')).toBeInTheDocument();
        expect(screen.getByText('Campaign #1')).toBeInTheDocument();
        expect(screen.getByText('Campaign #2')).toBeInTheDocument();
        expect(screen.getByText(/Sample data/)).toBeInTheDocument();
    });

    it('reports a timeframe change', () => {
        const onTimeframeChange = vi.fn();
        render(<AnalyticsDashboard campaigns={campaigns} timeframe="30d" onTimeframeChange={onTimeframeChange} />);

        fireEvent.click(screen.getByRole('button', { name: '7d' }));

        expect(onTimeframeChange).toHaveBeenCalledWith('7d');
    });
});
