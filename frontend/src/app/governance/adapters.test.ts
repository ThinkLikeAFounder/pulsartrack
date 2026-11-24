import { describe, expect, it } from 'vitest';
import { normalizeProposals, toGovernanceProposal, toVoteType } from './adapters';

describe('governance page adapters', () => {
  it('normalizes demo-store shaped proposals', () => {
    const [proposal] = normalizeProposals([
      {
        id: 3,
        title: 'Demo proposal',
        description: 'From the demo store',
        proposer: 'GDEMO',
        votes_for: 10,
        votes_against: 2,
        votes_abstain: 1,
        end_time: 1_700_000_000,
        is_active: true,
      },
    ]);

    expect(proposal.proposal_id).toBe(3n);
    expect(proposal.status).toBe('Active');
    expect(proposal.voting_ends_at).toBe(1_700_000_000n);
    expect(proposal.votes_for).toBe(10n);
  });

  it('prefers contract-shaped fields and falls back safely', () => {
    const proposal = toGovernanceProposal({
      proposal_id: '7',
      status: 'Executed',
      voting_ends_at: 42,
      created_at: 1,
    });

    expect(proposal.proposal_id).toBe(7n);
    expect(proposal.status).toBe('Executed');
    expect(proposal.voting_ends_at).toBe(42n);
    expect(proposal.created_at).toBe(1n);
    expect(proposal.votes_abstain).toBe(0n);
    expect(proposal.proposer).toBe('');
  });

  it('maps component votes to contract vote types', () => {
    expect(toVoteType('for')).toBe('For');
    expect(toVoteType('against')).toBe('Against');
    expect(toVoteType('abstain')).toBe('Abstain');
  });
});
