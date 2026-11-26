import type { GovernanceProposal } from '@/types/contracts';

type Scalar = bigint | number | string | null | undefined;

/**
 * Proposals arrive in different shapes: contract reads use the snake_case
 * struct fields, while the demo store uses `id`, `end_time` and `is_active`.
 * Everything is normalized before it reaches the shared governance components.
 */
export interface RawGovernanceProposal {
  id?: Scalar;
  proposal_id?: Scalar;
  status?: GovernanceProposal['status'];
  is_active?: boolean;
  voting_ends_at?: Scalar;
  end_time?: Scalar;
  end_ledger?: Scalar;
  created_at?: Scalar;
  proposer?: string | null;
  title?: string | null;
  description?: string | null;
  votes_for?: Scalar;
  votes_against?: Scalar;
  votes_abstain?: Scalar;
}

function toBigInt(value: Scalar, fallback = 0n): bigint {
  if (value === null || value === undefined || value === '') return fallback;
  try {
    return BigInt(value);
  } catch {
    return fallback;
  }
}

export function toGovernanceProposal(raw: RawGovernanceProposal): GovernanceProposal {
  return {
    proposal_id: toBigInt(raw.proposal_id ?? raw.id),
    proposer: raw.proposer ?? '',
    title: raw.title ?? '',
    description: raw.description ?? '',
    status: raw.status ?? (raw.is_active ? 'Active' : 'Passed'),
    votes_for: toBigInt(raw.votes_for),
    votes_against: toBigInt(raw.votes_against),
    votes_abstain: toBigInt(raw.votes_abstain),
    created_at: toBigInt(raw.created_at),
    voting_ends_at: toBigInt(raw.voting_ends_at ?? raw.end_time ?? raw.end_ledger),
  };
}

export function normalizeProposals(
  proposals: readonly RawGovernanceProposal[] | undefined,
): GovernanceProposal[] {
  return (proposals ?? []).map(toGovernanceProposal);
}

const VOTE_TYPES = {
  for: 'For',
  against: 'Against',
  abstain: 'Abstain',
} as const;

export type ProposalVote = keyof typeof VOTE_TYPES;

export function toVoteType(vote: ProposalVote): 'For' | 'Against' | 'Abstain' {
  return VOTE_TYPES[vote];
}
