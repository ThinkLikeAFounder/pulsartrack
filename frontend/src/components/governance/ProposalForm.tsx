'use client';

import { useState } from 'react';

interface ProposalFormProps {
  onSubmit?: (data: { title: string; description: string; durationDays: number }) => Promise<void>;
  onCancel?: () => void;
  requiredBalance?: number; // PULSAR tokens required
}

export function ProposalForm({ onSubmit, onCancel, requiredBalance = 1000 }: ProposalFormProps) {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [durationDays, setDurationDays] = useState(7);
  const [isPending, setIsPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    if (!title.trim()) { setError('Title is required'); return; }
    if (!description.trim() || description.length < 50) {
      setError('Description must be at least 50 characters');
      return;
    }

    setIsPending(true);
    try {
      await onSubmit?.({ title, description, durationDays });
      setTitle('');
      setDescription('');
    } catch (err: any) {
      setError(err?.message || 'Failed to submit proposal');
    } finally {
      setIsPending(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="bg-yellow-900/20 border border-yellow-700/50 rounded-lg px-4 py-3 text-sm text-yellow-300">
        Creating a proposal requires holding{' '}
        <span className="font-semibold">{requiredBalance.toLocaleString()} PULSAR</span> tokens.
        Proposals are binding and subject to timelock execution.
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">
          Proposal Title <span className="text-red-400">*</span>
        </label>
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="e.g. Reduce platform fee to 1.5%"
          maxLength={120}
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
        />
        <p className="text-xs text-gray-500 mt-1 text-right">{title.length}/120</p>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">
          Description <span className="text-red-400">*</span>
        </label>
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Describe the proposal in detail. Include motivation, implementation plan, and expected outcomes. Minimum 50 characters."
          rows={6}
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm resize-none"
        />
        <p className="text-xs text-gray-500 mt-1">
          {description.length} chars {description.length < 50 && `(need ${50 - description.length} more)`}
        </p>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-300 mb-1">
          Voting Duration
        </label>
        <select
          value={durationDays}
          onChange={(e) => setDurationDays(parseInt(e.target.value))}
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-indigo-500 text-sm"
        >
          {[3, 5, 7, 14].map((d) => (
            <option key={d} value={d}>{d} days</option>
          ))}
        </select>
      </div>

      {error && (
        <div className="bg-red-900/30 border border-red-700 rounded-lg px-3 py-2 text-red-300 text-sm">
          {error}
        </div>
      )}

      <div className="flex gap-3">
        <button
          type="submit"
          disabled={isPending}
          className="flex-1 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium py-2 px-4 rounded-lg transition-colors text-sm"
        >
          {isPending ? 'Submitting...' : 'Submit Proposal'}
        </button>
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            className="px-4 py-2 border border-gray-600 text-gray-300 rounded-lg hover:bg-gray-700 transition-colors text-sm"
          >
            Cancel
          </button>
        )}
      </div>
    </form>
  );
}
