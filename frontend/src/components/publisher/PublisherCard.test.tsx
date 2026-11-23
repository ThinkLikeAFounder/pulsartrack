import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { PublisherCard } from './PublisherCard';
import { EarningsDisplay } from './EarningsDisplay';
import { toPublisher } from '@/lib/contract-adapters';

const ADDR = 'GABCDEFGHIJKLMNOPQRSTUVWXYZ234567ABCDEFGHIJKLMNOPQRSTUVW';

describe('PublisherCard', () => {
    it('shows the publisher read from the chain', () => {
        const publisher = toPublisher(ADDR, {
            status: ['Verified'],
            domain: 'news.example',
            total_earnings: BigInt(55_000_000),
            total_impressions: BigInt(12_000),
            reputation_score: 720,
        });
        render(<PublisherCard publisher={publisher} />);

        expect(screen.getByRole('heading', { name: 'news.example' })).toBeInTheDocument();
        expect(screen.getByText('Verified')).toBeInTheDocument();
        expect(screen.getByText('12,000')).toBeInTheDocument();
        expect(screen.getByText('5.50 XLM')).toBeInTheDocument();
    });
});

describe('EarningsDisplay', () => {
    it('shows total earnings and the balance waiting to be paid', () => {
        render(<EarningsDisplay totalEarnings={BigInt(55_000_000)} pendingPayout={BigInt(20_000_000)} />);

        expect(screen.getByText('5.50 XLM')).toBeInTheDocument();
        expect(screen.getByText('2.00 XLM')).toBeInTheDocument();
        expect(screen.queryByRole('button', { name: 'Withdraw' })).not.toBeInTheDocument();
    });

    it('renders the on-chain revenue split instead of fixed percentages', () => {
        render(
            <EarningsDisplay
                totalEarnings={BigInt(0)}
                revenueSplit={{ publisherPct: 8_000, treasuryPct: 1_000, platformPct: 750, burnPct: 250 }}
            />,
        );

        expect(screen.getByText('80%')).toBeInTheDocument();
        expect(screen.getByText('10%')).toBeInTheDocument();
        expect(screen.getByText('7.5%')).toBeInTheDocument();
        expect(screen.getByText('2.5%')).toBeInTheDocument();
        expect(screen.queryByText('90%')).not.toBeInTheDocument();
    });

    it('does not show a split when it could not be read from the contract', () => {
        render(<EarningsDisplay totalEarnings={BigInt(0)} />);

        expect(screen.queryByText('90%')).not.toBeInTheDocument();
        expect(screen.getByText(/Revenue split unavailable/)).toBeInTheDocument();
    });
});
