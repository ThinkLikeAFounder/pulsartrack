'use client';

import { clsx } from 'clsx';
import { getReputationTier } from '@/lib/display-utils';

interface ReputationBadgeProps {
  score: number; // 0-1000
  size?: 'sm' | 'md' | 'lg';
  showScore?: boolean;
}

const TIER_STYLES: Record<string, { badge: string; ring: string; label: string }> = {
  Bronze: {
    badge: 'bg-amber-900/40 text-amber-400 border border-amber-700',
    ring: 'stroke-amber-600',
    label: '🥉 Bronze',
  },
  Silver: {
    badge: 'bg-slate-700/40 text-slate-300 border border-slate-500',
    ring: 'stroke-slate-400',
    label: '🥈 Silver',
  },
  Gold: {
    badge: 'bg-yellow-900/40 text-yellow-300 border border-yellow-600',
    ring: 'stroke-yellow-400',
    label: '🥇 Gold',
  },
  Platinum: {
    badge: 'bg-cyan-900/40 text-cyan-300 border border-cyan-600',
    ring: 'stroke-cyan-400',
    label: '💎 Platinum',
  },
};

function ScoreRing({ score, tier }: { score: number; tier: string }) {
  const r = 28;
  const circ = 2 * Math.PI * r;
  const filled = (score / 1000) * circ;
  const styles = TIER_STYLES[tier] || TIER_STYLES.Bronze;

  return (
    <svg
      width={72}
      height={72}
      viewBox="0 0 72 72"
      className="rotate-[-90deg]"
      role="img"
      aria-label={`Reputation score ${score} out of 1000, ${tier} tier`}
    >
      <circle cx={36} cy={36} r={r} fill="none" stroke="#374151" strokeWidth={6} />
      <circle
        cx={36}
        cy={36}
        r={r}
        fill="none"
        className={styles.ring}
        strokeWidth={6}
        strokeDasharray={`${filled} ${circ - filled}`}
        strokeLinecap="round"
      />
    </svg>
  );
}

export function ReputationBadge({ score, size = 'md', showScore = true }: ReputationBadgeProps) {
  const tier = getReputationTier(score);
  const styles = TIER_STYLES[tier] || TIER_STYLES.Bronze;

  if (size === 'sm') {
    return (
      <span className={clsx('text-xs font-medium px-2 py-0.5 rounded-full', styles.badge)}>
        {styles.label}
        {showScore && <span className="ml-1 opacity-70">{score}</span>}
      </span>
    );
  }

  if (size === 'lg') {
    return (
      <div className="flex flex-col items-center gap-2">
        <div className="relative">
          <ScoreRing score={score} tier={tier} />
          <div className="absolute inset-0 flex items-center justify-center rotate-90">
            <span className="text-lg font-bold text-white">{score}</span>
          </div>
        </div>
        <span className={clsx('text-sm font-semibold px-3 py-1 rounded-full', styles.badge)}>
          {styles.label}
        </span>
      </div>
    );
  }

  // md
  return (
    <div className="flex items-center gap-3">
      <div className="relative">
        <ScoreRing score={score} tier={tier} />
        <div className="absolute inset-0 flex items-center justify-center rotate-90">
          <span className="text-sm font-bold text-white">{score}</span>
        </div>
      </div>
      <div>
        <p className="text-white font-semibold">{score} / 1000</p>
        <span className={clsx('text-xs font-medium px-2 py-0.5 rounded-full', styles.badge)}>
          {styles.label}
        </span>
      </div>
    </div>
  );
}
