'use client';

import { GovernanceProposal } from '@/types/contracts';
import { formatAddress } from '@/lib/display-utils';
import { clsx } from 'clsx';

interface ProposalCardProps {
  proposal: GovernanceProposal;
  onVote?: (proposalId: number, vote: 'for' | 'against' | 'abstain') => void;
  isVoting?: boolean;
  userVote?: 'for' | 'against' | 'abstain' | null;
}

const STATUS_STYLES: Record<string, string> = {
  Active: 'bg-green-900/40 text-green-300 border border-green-700',
  Passed: 'bg-blue-900/40 text-blue-300 border border-blue-700',
  Failed: 'bg-red-900/40 text-red-300 border border-red-700',
  Executed: 'bg-purple-900/40 text-purple-300 border border-purple-700',
  Queued: 'bg-yellow-900/40 text-yellow-300 border border-yellow-700',
};

function VoteBar({ forVotes, againstVotes, abstainVotes }: {
  forVotes: bigint;
  againstVotes: bigint;
  abstainVotes: bigint;
}) {
  const total = forVotes + againstVotes + abstainVotes;
  if (total === BigInt(0)) {
    return (
      <div className="w-full h-3 bg-gray-700 rounded-full overflow-hidden">
        <div className="h-full w-full bg-gray-600" />
      </div>
    );
  }

  const forPct = Number((forVotes * BigInt(10000)) / total) / 100;
  const againstPct = Number((againstVotes * BigInt(10000)) / total) / 100;
  // abstain fills the rest

  return (
    <div className="w-full h-3 bg-gray-700 rounded-full overflow-hidden flex">
      <div
        className="h-full bg-green-500 transition-all"
        style={{ width: `${forPct}%` }}
      />
      <div
        className="h-full bg-gray-500 transition-all"
        style={{ width: `${100 - forPct - againstPct}%` }}
      />
      <div
        className="h-full bg-red-500 transition-all"
        style={{ width: `${againstPct}%` }}
      />
    </div>
  );
}

export function ProposalCard({ proposal, onVote, isVoting, userVote }: ProposalCardProps) {
  const total = proposal.votes_for + proposal.votes_against + proposal.votes_abstain;
  const forPct = total > BigInt(0)
    ? ((Number(proposal.votes_for) / Number(total)) * 100).toFixed(1)
    : '0.0';
  const isActive = proposal.status === 'Active';
  const deadline = new Date(Number(proposal.voting_ends_at) * 1000);
  const now = Date.now();
  const timeLeft = Number(proposal.voting_ends_at) * 1000 - now;
  const daysLeft = Math.max(0, Math.floor(timeLeft / 86400000));

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-xl p-5">
      {/* Header */}
      <div className="flex items-start justify-between gap-3 mb-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-xs text-gray-500 font-mono">PIP-{proposal.proposal_id}</span>
            <span
              className={clsx(
                'text-xs font-medium px-2 py-0.5 rounded-full',
                STATUS_STYLES[proposal.status] || 'bg-gray-700 text-gray-300'
              )}
            >
              {proposal.status}
            </span>
          </div>
          <h3 className="text-white font-semibold text-sm leading-snug">{proposal.title}</h3>
        </div>
        {isActive && daysLeft >= 0 && (
          <div className="flex-shrink-0 text-right">
            <p className="text-xs text-gray-500">Ends in</p>
            <p className="text-xs font-medium text-cyan-400">{daysLeft}d</p>
          </div>
        )}
      </div>

      {/* Description */}
      <p className="text-sm text-gray-400 mb-4 line-clamp-2">{proposal.description}</p>

      {/* Vote bar */}
      <VoteBar
        forVotes={proposal.votes_for}
        againstVotes={proposal.votes_against}
        abstainVotes={proposal.votes_abstain}
      />

      <div className="flex justify-between text-xs text-gray-500 mt-1.5 mb-4">
        <span className="text-green-400">{forPct}% For</span>
        <span className="text-gray-400">
          {Number(total).toLocaleString()} votes total
        </span>
        <span className="text-red-400">
          {total > BigInt(0)
            ? (100 - parseFloat(forPct)).toFixed(1)
            : '0.0'}% Against
        </span>
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between pt-3 border-t border-gray-700">
        <p className="text-xs text-gray-500">
          by <span className="text-gray-400 font-mono">{formatAddress(proposal.proposer)}</span>
        </p>

        {isActive && onVote && !userVote && (
          <div className="flex gap-1.5">
            {(['for', 'against', 'abstain'] as const).map((vote) => (
              <button
                key={vote}
                onClick={() => onVote(Number(proposal.proposal_id), vote)}
                disabled={isVoting}
                className={clsx(
                  'text-xs px-3 py-1.5 rounded-lg font-medium transition-colors disabled:opacity-50',
                  vote === 'for' && 'bg-green-800/50 hover:bg-green-700 text-green-300',
                  vote === 'against' && 'bg-red-800/50 hover:bg-red-700 text-red-300',
                  vote === 'abstain' && 'bg-gray-700 hover:bg-gray-600 text-gray-300'
                )}
              >
                {vote.charAt(0).toUpperCase() + vote.slice(1)}
              </button>
            ))}
          </div>
        )}

        {userVote && (
          <span className="text-xs text-gray-400">
            You voted:{' '}
            <span
              className={clsx(
                'font-medium',
                userVote === 'for' && 'text-green-400',
                userVote === 'against' && 'text-red-400',
                userVote === 'abstain' && 'text-gray-300'
              )}
            >
              {userVote}
            </span>
          </span>
        )}
      </div>
    </div>
  );
}
