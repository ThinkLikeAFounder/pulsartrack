import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { CampaignForm, parseCampaignSubmission } from './CampaignForm';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { useCreateCampaign } from '@/hooks/useContract';
import { campaignSchema } from '@/lib/validation/schemas';

// Mock the hook
vi.mock('@/hooks/useContract', () => ({
    useCreateCampaign: vi.fn(),
}));

describe('CampaignForm', () => {
    const mockCreateCampaign = vi.fn();
    const mockOnSuccess = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
        vi.mocked(useCreateCampaign).mockReturnValue({
            createCampaign: mockCreateCampaign,
            isPending: false,
        } as Partial<ReturnType<typeof useCreateCampaign>> as ReturnType<typeof useCreateCampaign>);
    });

    it('should show error if title is missing', async () => {
        render(<CampaignForm />);

        const submitButton = screen.getByText('Create Campaign');
        fireEvent.click(submitButton);

        expect(await screen.findByText(/Title is required/i)).toBeInTheDocument();
        expect(mockCreateCampaign).not.toHaveBeenCalled();
    });

    it('should reject campaign titles longer than the backend limit', () => {
        const result = campaignSchema.safeParse({
            title: 'x'.repeat(201),
            contentId: 'ipfs://123',
            campaignType: 1,
            budgetXlm: '100',
            costPerViewXlm: '0.01',
            durationDays: 30,
            targetViews: '100',
            dailyViewLimit: '10',
            refundable: true,
        });

        expect(result.success).toBe(false);
        if (!result.success) {
            expect(result.error.issues[0].message).toBe('Title must be 200 characters or less');
        }
    });

    it('should call createCampaign with correct parameters on valid submission', async () => {
        render(<CampaignForm onSuccess={mockOnSuccess} />);

        fireEvent.change(screen.getByLabelText(/Campaign Title/i), {
            target: { value: 'Test Campaign' },
        });
        fireEvent.change(screen.getByLabelText(/Content ID/i), {
            target: { value: 'ipfs://123' },
        });
        fireEvent.change(screen.getByLabelText(/Total Budget/i), {
            target: { value: '100' },
        });

        mockCreateCampaign.mockResolvedValue({ success: true, result: 1 });

        fireEvent.click(screen.getByText('Create Campaign'));

        await waitFor(() => {
            expect(mockCreateCampaign).toHaveBeenCalledWith(expect.objectContaining({
                title: 'Test Campaign',
                contentId: 'ipfs://123',
                budgetXlm: 100,
            }));
            expect(mockOnSuccess).toHaveBeenCalledWith(1);
        });
    });

    it('should reject non-finite numeric values before contract submission', () => {
        expect(parseCampaignSubmission({
            title: 'Overflow Campaign',
            contentId: 'ipfs://overflow',
            campaignType: 1,
            budgetXlm: '1e309',
            costPerViewXlm: '0.001',
            durationDays: 30,
            targetViews: '10000',
            dailyViewLimit: '1000',
            refundable: true,
        })).toEqual({
            ok: false,
            error: 'Invalid numeric values',
        });
    });

    it('should handle submission error', async () => {
        mockCreateCampaign.mockRejectedValue(new Error('Contract call failed'));

        render(<CampaignForm />);

        fireEvent.change(screen.getByLabelText(/Campaign Title/i), {
            target: { value: 'Error Campaign' },
        });
        fireEvent.change(screen.getByLabelText(/Content ID/i), {
            target: { value: 'error' },
        });
        fireEvent.change(screen.getByLabelText(/Total Budget/i), {
            target: { value: '10' },
        });

        fireEvent.click(screen.getByText('Create Campaign'));

        expect(await screen.findByText(/Contract call failed/i)).toBeInTheDocument();
    });
});
