'use client';

import { clsx } from 'clsx';

import { formatAddress } from '@/lib/display-utils';

export type FraudSeverity = 'low' | 'medium' | 'high' | 'critical';

export interface FraudAlertData {
  id: string;
  severity: FraudSeverity;
  type: string;
  address: string;
  description: string;
  timestamp: number;
  txHash?: string;
  resolved?: boolean;
}

interface FraudAlertProps {
  alert: FraudAlertData;
  onResolve?: (id: string) => void;
  onDismiss?: (id: string) => void;
}

const SEVERITY_STYLES: Record<FraudSeverity, { bg: string; border: string; text: string; badge: string; icon: string; label: string }> = {
  low: {
    bg: 'bg-yellow-900/10',
    border: 'border-yellow-800/40',
    text: 'text-yellow-300',
    badge: 'bg-yellow-900/40 text-yellow-300 border border-yellow-700',
    icon: '⚠',
    label: 'Low severity',
  },
  medium: {
    bg: 'bg-orange-900/10',
    border: 'border-orange-800/40',
    text: 'text-orange-300',
    badge: 'bg-orange-900/40 text-orange-300 border border-orange-700',
    icon: '⚠',
    label: 'Medium severity',
  },
  high: {
    bg: 'bg-red-900/15',
    border: 'border-red-800/50',
    text: 'text-red-300',
    badge: 'bg-red-900/40 text-red-300 border border-red-700',
    icon: '🚨',
    label: 'High severity',
  },
  critical: {
    bg: 'bg-red-900/25',
    border: 'border-red-600/70',
    text: 'text-red-200',
    badge: 'bg-red-700 text-white border border-red-500',
    icon: '🚨',
    label: 'Critical severity',
  },
};

export function FraudAlert({ alert, onResolve, onDismiss }: FraudAlertProps) {
  const styles = SEVERITY_STYLES[alert.severity];
  const time = new Date(alert.timestamp * 1000).toLocaleString();

  if (alert.resolved) {
    return (
      <div className="bg-gray-800/50 border border-gray-700/50 rounded-xl p-3 opacity-50">
        <div className="flex items-center justify-between">
          <span className="text-xs text-gray-500 line-through">{alert.type}</span>
          <span className="text-xs text-green-500">Resolved</span>
        </div>
      </div>
    );
  }

  return (
    <div className={clsx('border rounded-xl p-4', styles.bg, styles.border)} role="alert">
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-2 flex-1 min-w-0">
          <span className="text-lg flex-shrink-0" aria-hidden="true">{styles.icon}</span>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <span className={clsx('text-xs font-bold uppercase tracking-wide px-2 py-0.5 rounded-full', styles.badge)}>
                {styles.label}
              </span>
              <span className="text-xs font-medium text-gray-300">{alert.type}</span>
            </div>
            <p className={clsx('text-sm mt-1', styles.text)}>{alert.description}</p>
            <div className="flex items-center gap-3 mt-1.5 text-xs text-gray-500">
              <span className="font-mono">{formatAddress(alert.address)}</span>
              <span>·</span>
              <span>{time}</span>
              {alert.txHash && (
                <>
                  <span>·</span>
                  <span className="font-mono">{formatAddress(alert.txHash)}</span>
                </>
              )}
            </div>
          </div>
        </div>

        <div className="flex gap-1.5 flex-shrink-0">
          {onResolve && (
            <button
              onClick={() => onResolve(alert.id)}
              className="text-xs px-2.5 py-1 bg-green-800/40 hover:bg-green-700/50 text-green-300 rounded-lg transition-colors"
            >
              Resolve
            </button>
          )}
          {onDismiss && (
            <button
              onClick={() => onDismiss(alert.id)}
              aria-label="Dismiss alert"
              className="text-xs px-2.5 py-1 bg-gray-700 hover:bg-gray-600 text-gray-400 rounded-lg transition-colors"
            >
              ✕
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
