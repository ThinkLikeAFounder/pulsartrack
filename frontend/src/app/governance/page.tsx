'use client';

import { useState } from 'react';
import {
  Vote,
  Plus,
  Clock,
  CheckCircle,
  XCircle,
  BarChart2,
} from 'lucide-react';
import { useWalletStore } from '@/store/wallet-store';
import { WalletConnectButton } from '@/components/wallet/WalletModal';
import {
  useGovernanceBalance,
  useProposalCount,
  useGovernanceProposals,
  useCastVote,
  useCreateProposal,
} from '@/hooks/useContract';
import { stroopsToXlm } from '@/lib/stellar-config';

type ProposalStatus = 'Active' | 'Passed' | 'Rejected' | 'Executed';

const STATUS_COLORS: Record<ProposalStatus, string> = {
  Active: 'bg-blue-100 text-blue-700',
  Passed: 'bg-green-100 text-green-700',
  Rejected: 'bg-red-100 text-red-700',
  Executed: 'bg-gray-100 text-gray-700',
};

import { ErrorBoundary } from '@/components/ErrorBoundary';

export default function GovernancePage() {
  const { address, isConnected } = useWalletStore();
  const [activeTab, setActiveTab] = useState<
    'proposals' | 'create' | 'my_votes'
  >('proposals');
  const [proposalData, setProposalData] = useState({
    title: '',
    description: '',
    votingPeriodDays: 7,
  });

  // Fetch governance data
  const { data: balance, isLoading: balanceLoading } = useGovernanceBalance(
    address || '',
    isConnected,
  );
  const { data: proposalCount } = useProposalCount(isConnected);
  const { data: proposals, isLoading: proposalsLoading } =
    useGovernanceProposals(proposalCount, isConnected);
  const { castVote, isPending: voteLoading } = useCastVote();
  const { createProposal, isPending: createLoading } = useCreateProposal();

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
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
            {[
              {
                label: 'Your PULSAR Balance',
                value: balanceLoading
                  ? 'Loading...'
                  : `${stroopsToXlm(balance || 0n).toFixed(2)} PULSAR`,
                icon: BarChart2,
              },
              {
                label: 'Voting Power',
                value: balanceLoading
                  ? 'Loading...'
                  : stroopsToXlm(balance || 0n).toFixed(0),
                icon: Vote,
              },
              {
                label: 'Active Proposals',
                value: proposalCount ?? '0',
                icon: Clock,
              },
            ].map(({ label, value, icon: Icon }) => (
              <div
                key={label}
                className="bg-white p-4 rounded-xl border border-gray-200 flex items-center gap-3"
              >
                <Icon className="w-8 h-8 text-indigo-500" />
                <div>
                  <p className="text-sm text-gray-600">{label}</p>
                  <p className="text-xl font-bold text-gray-900">{value}</p>
                </div>
              </div>
            ))}
          </div>

          {/* Tabs */}
          <div className="flex gap-1 mb-6 bg-gray-100 p-1 rounded-lg w-fit">
            {[
              { id: 'proposals', label: 'Proposals' },
              { id: 'create', label: 'Create Proposal' },
              { id: 'my_votes', label: 'My Votes' },
            ].map(({ id, label }) => (
              <button
                key={id}
                onClick={() => setActiveTab(id as any)}
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
              ) : proposals && proposals.length > 0 ? (
                proposals.map((proposal) => {
                  const total =
                    (proposal.votes_for || 0n) +
                    (proposal.votes_against || 0n) +
                    (proposal.votes_abstain || 0n);
                  const forPct =
                    total > 0n
                      ? Number(((proposal.votes_for || 0n) * 100n) / total)
                      : 0;
                  const againstPct =
                    total > 0n
                      ? Number(((proposal.votes_against || 0n) * 100n) / total)
                      : 0;

                  return (
                    <div
                      key={proposal.id}
                      className="bg-white rounded-xl border border-gray-200 p-6"
                    >
                      <div className="flex items-start justify-between mb-3">
                        <div>
                          <h3 className="text-lg font-semibold text-gray-900">
                            {proposal.title}
                          </h3>
                          <p className="text-sm text-gray-500 mt-1">
                            By {proposal.proposer?.substring(0, 8)}...
                            {proposal.proposer?.substring(
                              proposal.proposer?.length - 4,
                            )}{' '}
                            &bull; Voting ends{' '}
                            {new Date(
                              Number(proposal.end_time) * 1000,
                            ).toLocaleDateString()}
                          </p>
                        </div>
                        <span
                          className={`px-2 py-1 rounded-full text-xs font-medium ${
                            proposal.is_active
                              ? STATUS_COLORS['Active']
                              : STATUS_COLORS['Passed']
                          }`}
                        >
                          {proposal.is_active ? 'Active' : 'Passed'}
                        </span>
                      </div>

                      <p className="text-gray-600 text-sm mb-4">
                        {proposal.description}
                      </p>

                      {/* Vote Bars */}
                      <div className="space-y-2 mb-4">
                        <div className="flex items-center gap-2">
                          <CheckCircle className="w-4 h-4 text-green-500" />
                          <div className="flex-1 bg-gray-100 rounded-full h-2">
                            <div
                              className="bg-green-500 rounded-full h-2 transition-all"
                              style={{ width: `${forPct}%` }}
                            />
                          </div>
                          <span className="text-xs text-gray-600 w-12 text-right">
                            {forPct.toFixed(1)}%
                          </span>
                        </div>
                        <div className="flex items-center gap-2">
                          <XCircle className="w-4 h-4 text-red-500" />
                          <div className="flex-1 bg-gray-100 rounded-full h-2">
                            <div
                              className="bg-red-500 rounded-full h-2 transition-all"
                              style={{ width: `${againstPct}%` }}
                            />
                          </div>
                          <span className="text-xs text-gray-600 w-12 text-right">
                            {againstPct.toFixed(1)}%
                          </span>
                        </div>
                      </div>

                      {proposal.is_active && (balance ?? 0n) > 0n && (
                        <div className="flex gap-3">
                          <button
                            onClick={() =>
                              castVote({
                                proposalId: proposal.id,
                                voteType: 'For',
                                votePower: balance || 0n,
                              })
                            }
                            disabled={voteLoading}
                            className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:bg-gray-400 transition-colors text-sm font-medium"
                          >
                            {voteLoading ? 'Voting...' : 'Vote For'}
                          </button>
                          <button
                            onClick={() =>
                              castVote({
                                proposalId: proposal.id,
                                voteType: 'Against',
                                votePower: balance || 0n,
                              })
                            }
                            disabled={voteLoading}
                            className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:bg-gray-400 transition-colors text-sm font-medium"
                          >
                            {voteLoading ? 'Voting...' : 'Vote Against'}
                          </button>
                          <button
                            onClick={() =>
                              castVote({
                                proposalId: proposal.id,
                                voteType: 'Abstain',
                                votePower: balance || 0n,
                              })
                            }
                            disabled={voteLoading}
                            className="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 disabled:bg-gray-100 transition-colors text-sm font-medium"
                          >
                            {voteLoading ? 'Voting...' : 'Abstain'}
                          </button>
                        </div>
                      )}
                      {!proposal.is_active && (
                        <p className="text-sm text-gray-500">
                          Voting period has ended
                        </p>
                      )}
                      {(balance ?? 0n) === 0n && (
                        <p className="text-sm text-gray-500">
                          You need PULSAR tokens to vote
                        </p>
                      )}
                    </div>
                  );
                })
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
              {(balance ?? 0n) === 0n ? (
                <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
                  <p className="text-sm text-yellow-800">
                    You need PULSAR tokens to create a proposal.
                  </p>
                </div>
              ) : (
                <form
                  className="space-y-4"
                  onSubmit={async (e) => {
                    e.preventDefault();
                    try {
                      await createProposal({
                        title: proposalData.title,
                        description: proposalData.description,
                        votingPeriodDays: proposalData.votingPeriodDays,
                      });
                      setProposalData({ title: '', description: '', votingPeriodDays: 7 });
                    } catch (error) {
                      console.error('Failed to create proposal:', error);
                    }
                  }}
                >
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Proposal Title
                    </label>
                    <input
                      type="text"
                      value={proposalData.title}
                      onChange={(e) =>
                        setProposalData({ ...proposalData, title: e.target.value })
                      }
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                      placeholder="A concise title for your proposal"
                      required
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Description
                    </label>
                    <textarea
                      value={proposalData.description}
                      onChange={(e) =>
                        setProposalData({ ...proposalData, description: e.target.value })
                      }
                      rows={4}
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                      placeholder="Detailed explanation of the proposed change and its rationale..."
                      required
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Voting Period (days)
                    </label>
                    <input
                      type="number"
                      value={proposalData.votingPeriodDays}
                      onChange={(e) =>
                        setProposalData({
                          ...proposalData,
                          votingPeriodDays: parseInt(e.target.value),
                        })
                      }
                      min={1}
                      max={30}
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                      required
                    />
                  </div>
                  <button
                    type="submit"
                    disabled={createLoading}
                    className="w-full py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:bg-gray-400 transition-colors font-medium"
                  >
                    {createLoading ? 'Creating...' : 'Submit Proposal'}
                  </button>
                  <p className="text-xs text-gray-500 text-center">
                  Requires minimum PULSAR token balance to submit proposals.
                </p>
              </form>
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
