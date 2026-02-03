'use client';

import { clsx } from 'clsx';

interface AudienceSegmentTagProps {
  segment: string;
  onRemove?: (segment: string) => void;
  variant?: 'active' | 'excluded' | 'neutral';
}

const VARIANT_STYLES = {
  active: 'bg-indigo-900/40 text-indigo-300 border border-indigo-700',
  excluded: 'bg-red-900/30 text-red-400 border border-red-800',
  neutral: 'bg-gray-700 text-gray-300 border border-gray-600',
};

export function AudienceSegmentTag({ segment, onRemove, variant = 'active' }: AudienceSegmentTagProps) {
  return (
    <span
      className={clsx(
        'inline-flex items-center gap-1.5 text-xs font-medium px-2.5 py-1 rounded-full',
        VARIANT_STYLES[variant]
      )}
    >
      {variant === 'excluded' && <span className="opacity-70">–</span>}
      {segment}
      {onRemove && (
        <button
          onClick={() => onRemove(segment)}
          className="opacity-60 hover:opacity-100 transition-opacity ml-0.5"
          aria-label={`Remove ${segment}`}
        >
          ×
        </button>
      )}
    </span>
  );
}

interface SegmentGroupProps {
  label: string;
  segments: string[];
  variant?: 'active' | 'excluded' | 'neutral';
  onRemove?: (segment: string) => void;
  onAdd?: (segment: string) => void;
  availableSegments?: string[];
}

export function SegmentGroup({
  label,
  segments,
  variant = 'active',
  onRemove,
  onAdd,
  availableSegments = [],
}: SegmentGroupProps) {
  const remaining = availableSegments.filter((s) => !segments.includes(s));

  return (
    <div>
      <p className="text-xs font-medium text-gray-400 uppercase tracking-wide mb-2">{label}</p>
      <div className="flex flex-wrap gap-1.5">
        {segments.map((seg) => (
          <AudienceSegmentTag
            key={seg}
            segment={seg}
            variant={variant}
            onRemove={onRemove}
          />
        ))}
        {onAdd && remaining.length > 0 && (
          <select
            onChange={(e) => { if (e.target.value) { onAdd(e.target.value); e.target.value = ''; } }}
            className="text-xs bg-gray-700 border border-gray-600 text-gray-400 rounded-full px-2 py-0.5 cursor-pointer focus:outline-none"
            defaultValue=""
          >
            <option value="" disabled>+ Add</option>
            {remaining.map((s) => (
              <option key={s} value={s}>{s}</option>
            ))}
          </select>
        )}
        {segments.length === 0 && <span className="text-xs text-gray-600 italic">None selected</span>}
      </div>
    </div>
  );
}
