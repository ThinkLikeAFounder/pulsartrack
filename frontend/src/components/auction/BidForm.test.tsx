import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BidForm } from './BidForm';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { usePlaceBid } from '@/hooks/useContract';
import { Auction } from '@/types/contracts';
import { demoContractCall } from '@/lib/demo-handlers';
import { resetDemoWorld, demoXlm } from '@/lib/demo-store';
import { addressToScVal, i128ToScVal, u64ToScVal } from '@/lib/soroban-client';

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

    // Submits the form directly (as the floor-price test does) so jsdom's
    // native min/step constraint check doesn't swallow the submit event.
    const submitBid = (amount: string, campaign = '123') => {
        const bidInput = screen.getByLabelText(/Bid Amount/i);
        fireEvent.change(bidInput, { target: { value: amount } });
        fireEvent.change(screen.getByLabelText(/Campaign ID/i), { target: { value: campaign } });
        fireEvent.submit(bidInput.closest('form')!);
    };

    // usePlaceBid throws this itself before any contract call is made.
    it('shows the hook error when the wallet is not connected', async () => {
        const onSuccess = vi.fn();
        mockMutateAsync.mockRejectedValue(new Error('Wallet not connected'));

        render(<BidForm auction={mockAuction} onSuccess={onSuccess} />);
        submitBid('5');

        expect(await screen.findByText('Wallet not connected')).toBeInTheDocument();
        expect(onSuccess).not.toHaveBeenCalled();
    });

    // callContract never throws for contract failures: it resolves with
    // `{ success: false, error }`. This is the exact shape it returns when
    // simulation rejects the transaction.
    it('shows a simulation failure returned by callContract', async () => {
        const onSuccess = vi.fn();
        mockMutateAsync.mockResolvedValue({
            success: false,
            error: 'Simulation failed: HostError: Error(Contract, #4)',
        });

        render(<BidForm auction={mockAuction} onSuccess={onSuccess} />);
        submitBid('5');

        expect(
            await screen.findByText('Simulation failed: HostError: Error(Contract, #4)'),
        ).toBeInTheDocument();
        expect(onSuccess).not.toHaveBeenCalled();
    });

    it('falls back to a generic message when a failed result has no error text', async () => {
        mockMutateAsync.mockResolvedValue({ success: false });

        render(<BidForm auction={mockAuction} />);
        submitBid('5');

        expect(await screen.findByText('Failed to place bid')).toBeInTheDocument();
    });

    // End-to-end against the real demo contract layer: the form was rendered
    // with a stale highest bid (2.4 XLM) while the simulated auction has since
    // moved to 3.2 XLM, so the form allows 3 XLM but place_bid rejects it.
    it('shows the "bid too low" rejection from the demo place_bid handler', async () => {
        window.localStorage.clear();
        resetDemoWorld();
        const bidder = 'GDEMOBIDDERAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';
        mockMutateAsync.mockImplementation((params: {
            auctionId: number;
            amountStroops: bigint;
            campaignId: number;
        }) =>
            demoContractCall({
                contractId: '',
                method: 'place_bid',
                source: bidder,
                args: [
                    addressToScVal(bidder),
                    u64ToScVal(params.auctionId),
                    i128ToScVal(params.amountStroops),
                    u64ToScVal(params.campaignId),
                ],
            }),
        );
        const onSuccess = vi.fn();
        const staleAuction: Auction = {
            ...mockAuction,
            auction_id: 1n,
            floor_price: demoXlm(2),
            winning_bid: demoXlm(2.4),
        };

        render(<BidForm auction={staleAuction} onSuccess={onSuccess} />);
        submitBid('3', '1');

        expect(await screen.findByText('bid too low', {}, { timeout: 4000 })).toBeInTheDocument();
        expect(onSuccess).not.toHaveBeenCalled();
    });

    it('calls onSuccess only when the bid is accepted', async () => {
        const onSuccess = vi.fn();
        mockMutateAsync.mockResolvedValue({ success: true, txHash: 'abc' });

        render(<BidForm auction={mockAuction} onSuccess={onSuccess} />);
        submitBid('5');

        await waitFor(() => expect(onSuccess).toHaveBeenCalledTimes(1));
        expect(screen.queryByText('Failed to place bid')).not.toBeInTheDocument();
    });
});

