'use client';

import { GovernanceProposal } from '@/types/contracts';

interface GovernanceStatsProps {
  proposals: GovernanceProposal[];
  tokenBalance?: bigint;
  votingPower?: bigint;
}

export function GovernanceStats({ proposals, tokenBalance = BigInt(0), votingPower = BigInt(0) }: GovernanceStatsProps) {
  const active = proposals.filter((p) => p.status === 'Active').length;
  const passed = proposals.filter((p) => p.status === 'Passed' || p.status === 'Executed').length;
  const failed = proposals.filter((p) => p.status === 'Rejected').length;

  const totalVotes = proposals.reduce(
    (acc, p) => acc + p.votes_for + p.votes_against + p.votes_abstain,
    BigInt(0)
  );

  const formatPulsar = (amount: bigint) => {
    return (Number(amount) / 1e7).toLocaleString(undefined, {
      minimumFractionDigits: 0,
      maximumFractionDigits: 2,
    });
  };

  const stats = [
    { label: 'Active Proposals', value: String(active), color: 'text-green-400' },
    { label: 'Passed', value: String(passed), color: 'text-blue-400' },
    { label: 'Failed', value: String(failed), color: 'text-red-400' },
    { label: 'Total Votes Cast', value: formatPulsar(totalVotes), color: 'text-purple-400' },
  ];

  return (
    <div className="space-y-4">
      {/* Voting power card */}
      <div className="bg-indigo-900/20 border border-indigo-700/50 rounded-xl p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xs text-indigo-300 font-medium uppercase tracking-wide">
              Your Voting Power
            </p>
            <p className="text-2xl font-bold text-white mt-1">
              {formatPulsar(votingPower)} PULSAR
            </p>
          </div>
          <div className="text-right">
            <p className="text-xs text-gray-400">Balance</p>
            <p className="text-sm text-gray-200 font-medium">
              {formatPulsar(tokenBalance)} PULSAR
            </p>
          </div>
        </div>
      </div>

      {/* Proposal stats */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        {stats.map(({ label, value, color }) => (
          <div key={label} className="bg-gray-800 border border-gray-700 rounded-xl p-3 text-center">
            <p className={`text-xl font-bold ${color}`}>{value}</p>
            <p className="text-xs text-gray-500 mt-0.5">{label}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
