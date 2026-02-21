'use client';

import { useState } from 'react';
import { useCreateCampaign } from '@/hooks/useContract';
import { xlmToStroops } from '@/lib/stellar-config';

interface CampaignFormProps {
  onSuccess?: (campaignId: number) => void;
  onCancel?: () => void;
}

const INITIAL_STATE = {
  title: '',
  contentId: '',
  budgetXlm: '',
  dailyBudgetXlm: '',
  durationDays: '30',
  targetGeo: '',
  targetInterests: '',
};

export function CampaignForm({ onSuccess, onCancel }: CampaignFormProps) {
  const [form, setForm] = useState(INITIAL_STATE);
  const [error, setError] = useState<string | null>(null);
  const { mutateAsync: createCampaign, isPending } = useCreateCampaign();

  const set = (field: keyof typeof form) => (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
  ) => setForm((f) => ({ ...f, [field]: e.target.value }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!form.title.trim()) { setError('Title is required'); return; }
    if (!form.contentId.trim()) { setError('Content ID is required'); return; }
    const budget = parseFloat(form.budgetXlm);
    if (isNaN(budget) || budget <= 0) { setError('Invalid budget'); return; }

    try {
      const result = await createCampaign({
        title: form.title,
        contentId: form.contentId,
        budget: xlmToStroops(budget),
        dailyBudget: xlmToStroops(parseFloat(form.dailyBudgetXlm) || budget / 30),
        durationDays: parseInt(form.durationDays) || 30,
      });
      onSuccess?.(result as number);
      setForm(INITIAL_STATE);
    } catch (err: any) {
      setError(err?.message || 'Failed to create campaign');
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label htmlFor="campaign-title" className="block text-sm font-medium text-gray-300 mb-1">
          Campaign Title <span className="text-red-400">*</span>
        </label>
        <input
          id="campaign-title"
          type="text"
          value={form.title}
          onChange={set('title')}
          placeholder="e.g. Summer Product Launch"
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
        />
      </div>

      <div>
        <label htmlFor="campaign-content-id" className="block text-sm font-medium text-gray-300 mb-1">
          Content ID <span className="text-red-400">*</span>
        </label>
        <input
          id="campaign-content-id"
          type="text"
          value={form.contentId}
          onChange={set('contentId')}
          placeholder="IPFS hash or content identifier"
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
        />
      </div>

      <div className="grid grid-cols-2 gap-3">
        <div>
          <label htmlFor="campaign-budget" className="block text-sm font-medium text-gray-300 mb-1">
            Total Budget (XLM) <span className="text-red-400">*</span>
          </label>
          <input
            id="campaign-budget"
            type="number"
            value={form.budgetXlm}
            onChange={set('budgetXlm')}
            placeholder="500"
            min="1"
            step="0.1"
            className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
          />
        </div>
        <div>
          <label htmlFor="campaign-daily-budget" className="block text-sm font-medium text-gray-300 mb-1">
            Daily Budget (XLM)
          </label>
          <input
            id="campaign-daily-budget"
            type="number"
            value={form.dailyBudgetXlm}
            onChange={set('dailyBudgetXlm')}
            placeholder="auto"
            min="1"
            step="0.1"
            className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
          />
        </div>
      </div>

      <div>
        <label htmlFor="campaign-duration" className="block text-sm font-medium text-gray-300 mb-1">
          Duration (days)
        </label>
        <select
          id="campaign-duration"
          value={form.durationDays}
          onChange={(e) => setForm((f) => ({ ...f, durationDays: e.target.value }))}
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-indigo-500 text-sm"
        >
          {[7, 14, 30, 60, 90].map((d) => (
            <option key={d} value={String(d)}>{d} days</option>
          ))}
        </select>
      </div>

      <div>
        <label htmlFor="campaign-geo" className="block text-sm font-medium text-gray-300 mb-1">
          Geographic Targets
        </label>
        <input
          id="campaign-geo"
          type="text"
          value={form.targetGeo}
          onChange={set('targetGeo')}
          placeholder="US,EU,APAC (comma-separated)"
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
        />
      </div>

      <div>
        <label htmlFor="campaign-interests" className="block text-sm font-medium text-gray-300 mb-1">
          Interest Segments
        </label>
        <input
          id="campaign-interests"
          type="text"
          value={form.targetInterests}
          onChange={set('targetInterests')}
          placeholder="tech,finance,gaming (comma-separated)"
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 text-sm"
        />
      </div>

      {error && (
        <div className="bg-red-900/30 border border-red-700 rounded-lg px-3 py-2 text-red-300 text-sm">
          {error}
        </div>
      )}

      <div className="flex gap-3 pt-2">
        <button
          type="submit"
          disabled={isPending}
          className="flex-1 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium py-2 px-4 rounded-lg transition-colors text-sm"
        >
          {isPending ? 'Creating...' : 'Create Campaign'}
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
