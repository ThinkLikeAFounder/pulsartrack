'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { SegmentGroup } from './AudienceSegmentTag';
import { targetingSchema, TargetingFormData } from '@/lib/validation/schemas';

const AVAILABLE_REGIONS = ['US', 'EU', 'UK', 'CA', 'AU', 'APAC', 'LATAM', 'MEA', 'GLOBAL'];
const AVAILABLE_INTERESTS = [
  'technology', 'finance', 'gaming', 'sports', 'entertainment',
  'health', 'travel', 'education', 'business', 'lifestyle',
];
const AVAILABLE_DEVICES = ['desktop', 'mobile', 'tablet', 'smart-tv'];
const AVAILABLE_LANGUAGES = ['en', 'es', 'fr', 'de', 'zh', 'ja', 'ko', 'pt', 'ar'];

export interface TargetingConfig {
  regions: string[];
  interests: string[];
  excludedSegments: string[];
  devices: string[];
  languages: string[];
  minAge: number;
  maxAge: number;
  minReputation: number;
  requireKyc: boolean;
  excludeFraud: boolean;
  maxCpmXlm: string;
}

interface TargetingFormProps {
  initial?: Partial<TargetingConfig>;
  onSave?: (config: TargetingConfig) => Promise<void>;
  isSaving?: boolean;
}

const DEFAULT_CONFIG: TargetingConfig = {
  regions: [],
  interests: [],
  excludedSegments: [],
  devices: ['desktop', 'mobile'],
  languages: ['en'],
  minAge: 18,
  maxAge: 65,
  minReputation: 0,
  requireKyc: false,
  excludeFraud: true,
  maxCpmXlm: '',
};

export function TargetingForm({ initial, onSave, isSaving }: TargetingFormProps) {
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    formState: { errors, isValid },
  } = useForm<TargetingFormData>({
    resolver: zodResolver(targetingSchema),
    mode: 'onTouched',
    defaultValues: { ...DEFAULT_CONFIG, ...initial },
  });

  const regions = watch('regions');
  const interests = watch('interests');
  const excludedSegments = watch('excludedSegments');
  const devices = watch('devices');
  const languages = watch('languages');

  const addTo = (field: 'regions' | 'interests' | 'excludedSegments' | 'devices' | 'languages') =>
    (val: string) => {
      const current = watch(field);
      setValue(field, [...current, val], { shouldValidate: true });
    };

  const removeFrom = (field: 'regions' | 'interests' | 'excludedSegments' | 'devices' | 'languages') =>
    (val: string) => {
      const current = watch(field);
      setValue(field, current.filter((v) => v !== val), { shouldValidate: true });
    };

  const onSubmit = async (data: TargetingFormData) => {
    setSubmitError(null);
    try {
      await onSave?.(data as TargetingConfig);
      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    } catch (err: any) {
      setSubmitError(err?.message || 'Failed to save targeting settings');
    }
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
      <div className="bg-gray-800 border border-gray-700 rounded-xl p-4 space-y-3">
        <h3 className="text-sm font-semibold text-gray-200">Geographic Targeting</h3>
        <SegmentGroup
          label="Target Regions"
          segments={regions}
          variant="active"
          onAdd={addTo('regions')}
          onRemove={removeFrom('regions')}
          availableSegments={AVAILABLE_REGIONS}
        />
      </div>

      <div className="bg-gray-800 border border-gray-700 rounded-xl p-4 space-y-3">
        <h3 className="text-sm font-semibold text-gray-200">Audience Segments</h3>
        <SegmentGroup
          label="Interest Segments"
          segments={interests}
          variant="active"
          onAdd={addTo('interests')}
          onRemove={removeFrom('interests')}
          availableSegments={AVAILABLE_INTERESTS}
        />
        <SegmentGroup
          label="Excluded Segments"
          segments={excludedSegments}
          variant="excluded"
          onAdd={addTo('excludedSegments')}
          onRemove={removeFrom('excludedSegments')}
          availableSegments={AVAILABLE_INTERESTS.filter((i) => !interests.includes(i))}
        />
      </div>

      <div className="bg-gray-800 border border-gray-700 rounded-xl p-4 space-y-3">
        <h3 className="text-sm font-semibold text-gray-200">Device & Language</h3>
        <SegmentGroup
          label="Device Types"
          segments={devices}
          variant="neutral"
          onAdd={addTo('devices')}
          onRemove={removeFrom('devices')}
          availableSegments={AVAILABLE_DEVICES}
        />
        <SegmentGroup
          label="Languages"
          segments={languages}
          variant="neutral"
          onAdd={addTo('languages')}
          onRemove={removeFrom('languages')}
          availableSegments={AVAILABLE_LANGUAGES}
        />
      </div>

      <div className="bg-gray-800 border border-gray-700 rounded-xl p-4">
        <h3 className="text-sm font-semibold text-gray-200 mb-3">Age Range</h3>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label htmlFor="targeting-min-age" className="block text-xs text-gray-400 mb-1">Min Age</label>
            <input
              id="targeting-min-age"
              type="number"
              {...register('minAge', { valueAsNumber: true })}
              min={13}
              max={100}
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-indigo-500"
            />
            {errors.minAge && (
              <p className="text-red-400 text-xs mt-1">{errors.minAge.message}</p>
            )}
          </div>
          <div>
            <label htmlFor="targeting-max-age" className="block text-xs text-gray-400 mb-1">Max Age</label>
            <input
              id="targeting-max-age"
              type="number"
              {...register('maxAge', { valueAsNumber: true })}
              min={13}
              max={100}
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-indigo-500"
            />
            {errors.maxAge && (
              <p className="text-red-400 text-xs mt-1">{errors.maxAge.message}</p>
            )}
          </div>
        </div>
      </div>

      <div className="bg-gray-800 border border-gray-700 rounded-xl p-4">
        <h3 className="text-sm font-semibold text-gray-200 mb-3">Quality Filters</h3>
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-300" id="exclude-fraud-label">Exclude Fraud</p>
              <p className="text-xs text-gray-500" id="exclude-fraud-desc">Block flagged addresses</p>
            </div>
            <input
              type="checkbox"
              {...register('excludeFraud')}
              aria-labelledby="exclude-fraud-label"
              aria-describedby="exclude-fraud-desc"
              className="w-4 h-4 accent-indigo-500"
            />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-300" id="require-kyc-label">Require KYC</p>
              <p className="text-xs text-gray-500" id="require-kyc-desc">Only verified users</p>
            </div>
            <input
              type="checkbox"
              {...register('requireKyc')}
              aria-labelledby="require-kyc-label"
              aria-describedby="require-kyc-desc"
              className="w-4 h-4 accent-indigo-500"
            />
          </div>
          <div>
            <label htmlFor="targeting-min-reputation" className="block text-xs text-gray-400 mb-1">Min Publisher Reputation (0-1000)</label>
            <input
              id="targeting-min-reputation"
              type="number"
              {...register('minReputation', { valueAsNumber: true })}
              min={0}
              max={1000}
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-indigo-500"
            />
            {errors.minReputation && (
              <p className="text-red-400 text-xs mt-1">{errors.minReputation.message}</p>
            )}
          </div>
          <div>
            <label htmlFor="targeting-max-cpm" className="block text-xs text-gray-400 mb-1">Max CPM (XLM, leave blank for no limit)</label>
            <input
              id="targeting-max-cpm"
              type="number"
              {...register('maxCpmXlm')}
              placeholder="0.001"
              min="0"
              step="0.0001"
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-500 text-sm focus:outline-none focus:border-indigo-500"
            />
            {errors.maxCpmXlm && (
              <p className="text-red-400 text-xs mt-1">{errors.maxCpmXlm.message}</p>
            )}
          </div>
        </div>
      </div>

      {submitError && (
        <div className="bg-red-900/30 border border-red-700 rounded-lg px-3 py-2 text-red-300 text-sm">
          {submitError}
        </div>
      )}

      {saved && (
        <div className="bg-green-900/30 border border-green-700 rounded-lg px-3 py-2 text-green-300 text-sm">
          Targeting settings saved!
        </div>
      )}

      <button
        type="submit"
        disabled={!isValid || isSaving}
        className="w-full bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium py-2 px-4 rounded-lg transition-colors text-sm"
      >
        {isSaving ? 'Saving...' : 'Save Targeting Settings'}
      </button>
    </form>
  );
}
