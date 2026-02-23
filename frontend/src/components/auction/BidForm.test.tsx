import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BidForm } from './BidForm';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { usePlaceBid } from '@/hooks/useContract';
import { Auction } from '@/types/contracts';

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
            mutateAsync: mockMutateAsync,
            isPending: false,
        } as any);
    });

    it('should render floor price correctly', () => {
        render(<BidForm auction={mockAuction} />);
        expect(screen.getByText('1 XLM')).toBeDefined();
    });

    it('should show error if bid is below floor price', async () => {
        render(<BidForm auction={mockAuction} />);

        const bidInput = screen.getByLabelText(/Bid Amount/i);
        const campaignInput = screen.getByLabelText(/Campaign ID/i);
        const submitButton = screen.getByText('Submit Bid');

        fireEvent.change(bidInput, { target: { value: '0.5' } });
        fireEvent.change(campaignInput, { target: { value: '10' } });
        fireEvent.click(submitButton);

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
                auctionId: 1n,
                campaignId: 123,
                amountStroops: 25000000n, // Matches BidForm call to placeBid
            });
        });
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
