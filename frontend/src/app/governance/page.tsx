'use client';

import { useState } from 'react';
import { Vote, Plus, Clock, CheckCircle, XCircle, BarChart2 } from 'lucide-react';
import { useWalletStore } from '@/store/wallet-store';
import { WalletConnectButton } from '@/components/wallet/WalletModal';

type ProposalStatus = 'Active' | 'Passed' | 'Rejected' | 'Executed';

interface MockProposal {
  id: number;
  title: string;
  description: string;
  status: ProposalStatus;
  votesFor: number;
  votesAgainst: number;
  votesAbstain: number;
  endsAt: string;
  proposer: string;
}

const MOCK_PROPOSALS: MockProposal[] = [
  {
    id: 1,
    title: 'Reduce Platform Fee to 2%',
    description: 'Proposal to reduce the platform fee from 2.5% to 2% to attract more advertisers.',
    status: 'Active',
    votesFor: 125000,
    votesAgainst: 45000,
    votesAbstain: 10000,
    endsAt: '2026-02-25',
    proposer: 'GAAZI4...WVNCF',
  },
  {
    id: 2,
    title: 'Add Support for USDC Payments',
    description: 'Integrate Circle USDC as an accepted payment token for ad campaigns.',
    status: 'Passed',
    votesFor: 200000,
    votesAgainst: 30000,
    votesAbstain: 5000,
    endsAt: '2026-02-10',
    proposer: 'GBVFY7...XKPQR',
  },
];

const STATUS_COLORS: Record<ProposalStatus, string> = {
  Active: 'bg-blue-100 text-blue-700',
  Passed: 'bg-green-100 text-green-700',
  Rejected: 'bg-red-100 text-red-700',
  Executed: 'bg-gray-100 text-gray-700',
};

import { ErrorBoundary } from '@/components/ErrorBoundary';

export default function GovernancePage() {
  const { address, isConnected } = useWalletStore();
  const [activeTab, setActiveTab] = useState<'proposals' | 'create' | 'my_votes'>('proposals');

  if (!isConnected) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center max-w-md p-8 bg-white rounded-2xl shadow-sm border border-gray-200">
          <div className="w-16 h-16 bg-indigo-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <Vote className="w-8 h-8 text-indigo-600" />
          </div>
          <h2 className="text-2xl font-bold text-gray-900 mb-2">Governance</h2>
          <p className="text-gray-600 mb-6">
            Connect your wallet to participate in PulsarTrack DAO governance with PULSAR tokens.
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
            <h1 className="text-2xl font-bold text-gray-900">PulsarTrack Governance</h1>
            <p className="text-sm text-gray-500 mt-1">
              Vote on proposals using your PULSAR tokens. Shape the future of the platform.
            </p>
          </div>
        </div>

        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          {/* Stats */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
            {[
              { label: 'Your PULSAR Balance', value: '0 PULSAR', icon: BarChart2 },
              { label: 'Voting Power', value: '0', icon: Vote },
              { label: 'Active Proposals', value: '1', icon: Clock },
            ].map(({ label, value, icon: Icon }) => (
              <div key={label} className="bg-white p-4 rounded-xl border border-gray-200 flex items-center gap-3">
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
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${activeTab === id
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
              {MOCK_PROPOSALS.map((proposal) => {
                const total = proposal.votesFor + proposal.votesAgainst + proposal.votesAbstain;
                const forPct = total > 0 ? (proposal.votesFor / total) * 100 : 0;
                const againstPct = total > 0 ? (proposal.votesAgainst / total) * 100 : 0;

                return (
                  <div key={proposal.id} className="bg-white rounded-xl border border-gray-200 p-6">
                    <div className="flex items-start justify-between mb-3">
                      <div>
                        <h3 className="text-lg font-semibold text-gray-900">{proposal.title}</h3>
                        <p className="text-sm text-gray-500 mt-1">
                          By {proposal.proposer} &bull; Ends {proposal.endsAt}
                        </p>
                      </div>
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${STATUS_COLORS[proposal.status]}`}>
                        {proposal.status}
                      </span>
                    </div>

                    <p className="text-gray-600 text-sm mb-4">{proposal.description}</p>

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
                        <span className="text-xs text-gray-600 w-12 text-right">{forPct.toFixed(1)}%</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <XCircle className="w-4 h-4 text-red-500" />
                        <div className="flex-1 bg-gray-100 rounded-full h-2">
                          <div
                            className="bg-red-500 rounded-full h-2 transition-all"
                            style={{ width: `${againstPct}%` }}
                          />
                        </div>
                        <span className="text-xs text-gray-600 w-12 text-right">{againstPct.toFixed(1)}%</span>
                      </div>
                    </div>

                    {proposal.status === 'Active' && (
                      <div className="flex gap-3">
                        <button className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors text-sm font-medium">
                          Vote For
                        </button>
                        <button className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm font-medium">
                          Vote Against
                        </button>
                        <button className="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors text-sm font-medium">
                          Abstain
                        </button>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}

          {activeTab === 'create' && (
            <div className="bg-white rounded-xl border border-gray-200 p-6 max-w-2xl">
              <h2 className="text-lg font-semibold text-gray-900 mb-6">Create Governance Proposal</h2>
              <form className="space-y-4" onSubmit={(e) => e.preventDefault()}>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Proposal Title</label>
                  <input
                    type="text"
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                    placeholder="A concise title for your proposal"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
                  <textarea
                    rows={4}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                    placeholder="Detailed explanation of the proposed change and its rationale..."
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Voting Period (days)</label>
                  <input
                    type="number"
                    defaultValue={7}
                    min={1}
                    max={30}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
                  />
                </div>
                <button
                  type="submit"
                  className="w-full py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors font-medium"
                >
                  Submit Proposal
                </button>
                <p className="text-xs text-gray-500 text-center">
                  Requires minimum PULSAR token balance to submit proposals.
                </p>
              </form>
            </div>
          )}

          {activeTab === 'my_votes' && (
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <h2 className="text-lg font-semibold text-gray-900 mb-4">My Voting History</h2>
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
