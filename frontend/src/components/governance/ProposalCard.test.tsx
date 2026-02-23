import { render, screen, fireEvent } from '@testing-library/react';
import { ProposalCard } from './ProposalCard';
import { vi, describe, it, expect } from 'vitest';
import { GovernanceProposal } from '@/types/contracts';

const mockProposal: GovernanceProposal = {
    proposal_id: 1n,
    proposer: 'GABC...123',
    title: 'Upgrade Protocol',
    description: 'A detailed description of the upgrade.',
    status: 'Active',
    votes_for: 1000n,
    votes_against: 500n,
    votes_abstain: 100n,
    created_at: 0n,
    voting_ends_at: BigInt(Math.floor(Date.now() / 1000) + 86400 * 3), // 3 days left
};

describe('ProposalCard', () => {
    it('should render proposal details correctly', () => {
        render(<ProposalCard proposal={mockProposal} />);

        expect(screen.getByText('PIP-1')).toBeDefined();
        expect(screen.getByText('Upgrade Protocol')).toBeDefined();
        expect(screen.getByText('Active')).toBeDefined();
        expect(screen.getByText(/62.5% For/i)).toBeDefined(); // 1000 / (1000+500+100) = 0.625
    });

    it('should call onVote when a vote button is clicked', () => {
        const mockOnVote = vi.fn();
        render(<ProposalCard proposal={mockProposal} onVote={mockOnVote} />);

        const forButton = screen.getByText('For');
        fireEvent.click(forButton);

        expect(mockOnVote).toHaveBeenCalledWith(1n, 'for');
    });

    it('should show user vote if already voted', () => {
        render(<ProposalCard proposal={mockProposal} userVote="against" />);

        expect(screen.getByText(/You voted:/i)).toBeDefined();
        expect(screen.getByText('against')).toBeDefined();
        expect(screen.queryByText('For')).toBeNull(); // Buttons should be hidden
    });

    it('should disable buttons when isVoting is true', () => {
        const mockOnVote = vi.fn();
        render(<ProposalCard proposal={mockProposal} onVote={mockOnVote} isVoting={true} />);

        const forButton = screen.getByText('For');
        expect(forButton).toBeInTheDocument();
        expect(forButton).toBeDisabled();
    });
});
