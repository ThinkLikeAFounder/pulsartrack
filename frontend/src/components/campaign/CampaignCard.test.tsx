import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { CampaignCard } from './CampaignCard';
import { CampaignStats } from './CampaignStats';
import { toCampaign } from '@/lib/contract-adapters';

const campaigns = [
    toCampaign({ id: 1, budget: BigInt(1_000_000_000), remaining_budget: BigInt(600_000_000), current_views: BigInt(3_000), status: ['Active'] } as never),
    toCampaign({ id: 2, budget: BigInt(500_000_000), remaining_budget: BigInt(500_000_000), current_views: BigInt(0), status: ['Paused'] } as never),
];

describe('CampaignCard', () => {
    it('shows a campaign\'s status, budget use and views', () => {
        render(<CampaignCard campaign={campaigns[0]} />);

        expect(screen.getByText('Campaign #1')).toBeInTheDocument();
        expect(screen.getByText('Active')).toBeInTheDocument();
        expect(screen.getByText('40%')).toBeInTheDocument();
        expect(screen.getByText('3,000')).toBeInTheDocument();
    });
});

describe('CampaignStats', () => {
    it('sums the advertiser\'s campaigns', () => {
        render(<CampaignStats campaigns={campaigns} />);

        expect(screen.getByText('of 2 total')).toBeInTheDocument();
        expect(screen.getByText('150.00 XLM')).toBeInTheDocument();
        expect(screen.getByText('40.00 XLM')).toBeInTheDocument();
    });

    it('shows zeros for an advertiser with no campaigns', () => {
        render(<CampaignStats campaigns={[]} />);

        expect(screen.getByText('of 0 total')).toBeInTheDocument();
        expect(screen.getByText('0.00% CTR')).toBeInTheDocument();
    });
});
