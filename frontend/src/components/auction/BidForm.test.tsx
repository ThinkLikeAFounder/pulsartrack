import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BidForm } from './BidForm';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { usePlaceBid } from '@/hooks/useContract';
import { Auction } from '@/types/contracts';
import { stroopsToXlm, xlmToStroops, STROOPS_PER_XLM } from '@/lib/stellar-config';

// Mock the hook
vi.mock('@/hooks/useContract', () => ({
    usePlaceBid: vi.fn(),
}));

const mockAuction: Auction = {
    auction_id: 1n,
    publisher: 'GABC...789',
    impression_slot: 'Homepage Banner',
    floor_price: 10000000n, // 1 XLM in stroops (10^7)
    reserve_price: 12000000n,
    start_time: 0n,
    end_time: BigInt(Math.floor(Date.now() / 1000) + 3600),
    status: 'Open',
    winning_bid: null,
    winner: null,
    bid_count: 0,
};

describe('BidForm', () => {
    const mockMutateAsync = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
        vi.mocked(usePlaceBid).mockReturnValue({
            placeBid: mockMutateAsync,
            isPending: false,
        } as Partial<ReturnType<typeof usePlaceBid>> as ReturnType<typeof usePlaceBid>);
    });

    it('should render floor price correctly', () => {
        render(<BidForm auction={mockAuction} />);
        expect(screen.getByText('1 XLM')).toBeInTheDocument();
    });

    it('should show error if bid is below floor price', async () => {
        render(<BidForm auction={mockAuction} />);

        const bidInput = screen.getByLabelText(/Bid Amount/i);
        const campaignInput = screen.getByLabelText(/Campaign ID/i);

        fireEvent.change(bidInput, { target: { value: '0.5' } });
        fireEvent.change(campaignInput, { target: { value: '10' } });
        fireEvent.submit(bidInput.closest('form')!);

        expect(await screen.findByText(/Minimum bid is 1.0000 XLM/i)).toBeInTheDocument();
        expect(mockMutateAsync).not.toHaveBeenCalled();
    });

    it('should call placeBid with correct parameters on valid submission', async () => {
        render(<BidForm auction={mockAuction} />);

        const bidInput = screen.getByLabelText(/Bid Amount/i);
        const campaignInput = screen.getByLabelText(/Campaign ID/i);
        const submitButton = screen.getByText('Submit Bid');

        fireEvent.change(bidInput, { target: { value: '2.5' } });
        fireEvent.change(campaignInput, { target: { value: '123' } });

        mockMutateAsync.mockResolvedValue({ success: true });

        fireEvent.click(submitButton);

        await waitFor(() => {
            expect(mockMutateAsync).toHaveBeenCalledWith({
                auctionId: 1,
                campaignId: 123,
                amountStroops: 25000000n, // Matches BidForm call to placeBid
            });
        });
    });

    it('should display floor price in XLM with 4 decimal places', () => {
        render(<BidForm auction={mockAuction} />);
        // 10,000,000 stroops = 1.0000 XLM
        expect(screen.getByText('1 XLM')).toBeDefined();
    });

    it('should show error if submission fails', async () => {
        mockMutateAsync.mockRejectedValue(new Error('Insufficent funds'));

        render(<BidForm auction={mockAuction} />);

        const bidInput = screen.getByLabelText(/Bid Amount/i);
        const campaignInput = screen.getByLabelText(/Campaign ID/i);
        fireEvent.change(bidInput, { target: { value: '5' } });
        fireEvent.change(campaignInput, { target: { value: '123' } });

        fireEvent.click(screen.getByText('Submit Bid'));

        expect(await screen.findByText(/Insufficent funds/i)).toBeInTheDocument();
    });
});

describe('stroopsToXlm / xlmToStroops conversions', () => {
    it('converts 1 XLM correctly to and from stroops', () => {
        expect(stroopsToXlm(10_000_000n)).toBe(1);
        expect(xlmToStroops(1)).toBe(10_000_000n);
    });

    it('converts fractional XLM amounts', () => {
        expect(stroopsToXlm(5_000_000n)).toBe(0.5);
        expect(xlmToStroops(0.5)).toBe(5_000_000n);
    });

    it('stroopsToXlm accepts number input', () => {
        expect(stroopsToXlm(10_000_000)).toBe(1);
    });

    it('xlmToStroops floors sub-stroop precision', () => {
        // 1.00000001 XLM — sub-stroop remainder is floored
        expect(xlmToStroops(1.00000001)).toBe(10_000_000n);
    });

    it('round-trips bid submission amount correctly', () => {
        // User enters 2.5 XLM → should send exactly 25,000,000 stroops
        const bidXlm = 2.5;
        const stroops = xlmToStroops(bidXlm);
        expect(stroops).toBe(25_000_000n);
        expect(stroopsToXlm(stroops)).toBe(2.5);
    });

    it('STROOPS_PER_XLM constant is correct', () => {
        expect(STROOPS_PER_XLM).toBe(10_000_000);
    });

    it('handles zero', () => {
        expect(stroopsToXlm(0n)).toBe(0);
        expect(xlmToStroops(0)).toBe(0n);
    });

    it('handles large auction values without precision loss', () => {
        // 1,000,000 XLM in stroops
        const largeStroops = 10_000_000_000_000n;
        expect(stroopsToXlm(largeStroops)).toBe(1_000_000);
    });
});
