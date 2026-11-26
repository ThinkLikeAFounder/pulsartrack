'use client';

import { useState } from 'react';
import { Vote } from 'lucide-react';
import { useWalletStore } from '@/store/wallet-store';
import { WalletConnectButton } from '@/components/wallet/WalletModal';
import {
  useGovernanceBalance,
  useProposalCount,
  useGovernanceProposals,
  useCastVote,
  useCreateProposal,
} from '@/hooks/useContract';
import { useToast } from '@/contexts/ToastContext';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import { GovernanceStats } from '@/components/governance/GovernanceStats';
import { ProposalCard } from '@/components/governance/ProposalCard';
import { ProposalForm } from '@/components/governance/ProposalForm';
import {
  normalizeProposals,
  toVoteType,
  type ProposalVote,
} from './adapters';

type GovernanceTab = 'proposals' | 'create' | 'my_votes';

const DURATION_OPTIONS = Array.from({ length: 30 }, (_, index) => index + 1);

export default function GovernancePage() {
  const { address, isConnected } = useWalletStore();
  const [activeTab, setActiveTab] = useState<GovernanceTab>('proposals');

  // Fetch governance data
  const { data: balance, isLoading: balanceLoading } = useGovernanceBalance(
    address || '',
    isConnected,
  );
  const { data: proposalCount } = useProposalCount(isConnected);
  const { data: proposals, isLoading: proposalsLoading } =
    useGovernanceProposals(proposalCount, isConnected);
  const { success, error: toastError } = useToast();
  const { castVote, isPending: voteLoading } = useCastVote();
  const { createProposal } = useCreateProposal();

  const normalizedProposals = normalizeProposals(proposals);
  const hasBalance = (balance ?? 0n) > 0n;

  if (!isConnected) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center max-w-md p-8 bg-white rounded-2xl shadow-sm border border-gray-200">
          <div className="w-16 h-16 bg-indigo-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <Vote className="w-8 h-8 text-indigo-600" />
          </div>
          <h2 className="text-2xl font-bold text-gray-900 mb-2">Governance</h2>
          <p className="text-gray-600 mb-6">
            Connect your wallet to participate in PulsarTrack DAO governance
            with PULSAR tokens.
          </p>
          <WalletConnectButton />
        </div>
      </div>
    );
  }

  const handleVote = (proposalId: number, vote: ProposalVote) => {
    void castVote({
      proposalId,
      voteType: toVoteType(vote),
      votePower: balance ?? 0n,
    });
  };

  return (
    <ErrorBoundary name="GovernancePage" resetKeys={[activeTab]}>
      <div className="min-h-screen bg-gray-50">
        <div className="bg-white border-b border-gray-200 px-4 sm:px-6 lg:px-8 py-6">
          <div className="max-w-7xl mx-auto">
            <h1 className="text-2xl font-bold text-gray-900">
              PulsarTrack Governance
            </h1>
            <p className="text-sm text-gray-500 mt-1">
              Vote on proposals using your PULSAR tokens. Shape the future of
              the platform.
            </p>
          </div>
        </div>

        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          {/* Stats */}
          {balanceLoading ? (
            <div className="bg-white rounded-xl border border-gray-200 p-6 text-center mb-6">
              <p className="text-gray-600">Loading governance data...</p>
            </div>
          ) : (
            <div className="mb-6">
              <GovernanceStats
                proposals={normalizedProposals}
                tokenBalance={balance ?? 0n}
                votingPower={balance ?? 0n}
              />
            </div>
          )}

          {/* Tabs */}
          <div className="flex gap-1 mb-6 bg-gray-100 p-1 rounded-lg w-fit">
            {([
              { id: 'proposals', label: 'Proposals' },
              { id: 'create', label: 'Create Proposal' },
              { id: 'my_votes', label: 'My Votes' },
            ] as Array<{ id: GovernanceTab; label: string }>).map(({ id, label }) => (
              <button
                key={id}
                onClick={() => setActiveTab(id)}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  activeTab === id
                    ? 'bg-white text-indigo-600 shadow-sm'
                    : 'text-gray-600 hover:text-gray-900'
                }`}
              >
                {label}
              </button>
            ))}
          </div>

          {activeTab === 'proposals' && (
            <div className="space-y-4">
              {proposalsLoading ? (
                <div className="bg-white rounded-xl border border-gray-200 p-6 text-center">
                  <p className="text-gray-600">Loading proposals...</p>
                </div>
              ) : normalizedProposals.length > 0 ? (
                normalizedProposals.map((proposal) => (
                  <div key={proposal.proposal_id.toString()}>
                    <ProposalCard
                      proposal={proposal}
                      isVoting={voteLoading}
                      onVote={proposal.status === 'Active' && hasBalance ? handleVote : undefined}
                    />
                    {proposal.status !== 'Active' && (
                      <p className="text-sm text-gray-500 mt-2">
                        Voting period has ended
                      </p>
                    )}
                    {!hasBalance && (
                      <p className="text-sm text-gray-500 mt-2">
                        You need PULSAR tokens to vote
                      </p>
                    )}
                  </div>
                ))
              ) : (
                <div className="bg-white rounded-xl border border-gray-200 p-6 text-center">
                  <p className="text-gray-600">No proposals yet</p>
                </div>
              )}
            </div>
          )}

          {activeTab === 'create' && (
            <div className="bg-white rounded-xl border border-gray-200 p-6 max-w-2xl">
              <h2 className="text-lg font-semibold text-gray-900 mb-6">
                Create Governance Proposal
              </h2>
              {!hasBalance ? (
                <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
                  <p className="text-sm text-yellow-800">
                    You need PULSAR tokens to create a proposal.
                  </p>
                </div>
              ) : (
                <ProposalForm
                  durationOptions={DURATION_OPTIONS}
                  onSubmit={async ({ title, description, durationDays }) => {
                    try {
                      await createProposal({
                        title,
                        description,
                        votingPeriodDays: durationDays,
                      });
                      success(
                        'Proposal submitted',
                        'Your governance proposal has been created.',
                      );
                    } catch (error) {
                      console.error('Failed to create proposal:', error);
                      toastError(
                        'Proposal submission failed',
                        error instanceof Error ? error.message : 'Please try again.',
                      );
                      // Let ProposalForm keep the entered values visible for a retry.
                      throw error;
                    }
                  }}
                />
              )}
            </div>
          )}

          {activeTab === 'my_votes' && (
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <h2 className="text-lg font-semibold text-gray-900 mb-4">
                My Voting History
              </h2>
              <div className="text-center py-12 text-gray-500">
                <Vote className="w-12 h-12 mx-auto mb-3 opacity-30" />
                <p>You have not voted on any proposals yet.</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </ErrorBoundary>
  );
}
