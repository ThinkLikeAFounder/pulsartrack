import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { CampaignForm } from './CampaignForm';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { useCreateCampaign } from '@/hooks/useContract';

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
            mutateAsync: mockCreateCampaign,
            isPending: false,
        } as any);
    });

    it('should show error if title is missing', async () => {
        render(<CampaignForm />);

        const submitButton = screen.getByText('Create Campaign');
        fireEvent.click(submitButton);

        expect(await screen.findByText(/Title is required/i)).toBeInTheDocument();
        expect(mockCreateCampaign).not.toHaveBeenCalled();
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

        mockCreateCampaign.mockResolvedValue(1); // Returns campaign ID

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
