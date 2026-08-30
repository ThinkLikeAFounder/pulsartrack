'use client';

import { FraudAlertData, FraudSeverity } from './FraudAlert';
import { FraudAlert } from './FraudAlert';
import { clsx } from 'clsx';

interface FraudStatsProps {
  alerts: FraudAlertData[];
  onResolve?: (id: string) => void;
  onDismiss?: (id: string) => void;
}

const SEVERITY_ORDER: FraudSeverity[] = ['critical', 'high', 'medium', 'low'];

export function FraudStats({ alerts, onResolve, onDismiss }: FraudStatsProps) {
  const active = alerts.filter((a) => !a.resolved);
  const resolved = alerts.filter((a) => a.resolved);

  const countBySeverity = SEVERITY_ORDER.reduce((acc, sev) => {
    acc[sev] = active.filter((a) => a.severity === sev).length;
    return acc;
  }, {} as Record<FraudSeverity, number>);

  const severityColors: Record<FraudSeverity, string> = {
    critical: 'text-red-300',
    high: 'text-red-400',
    medium: 'text-orange-400',
    low: 'text-yellow-400',
  };

  return (
    <div className="space-y-4">
      {/* Summary row */}
      <div className="grid grid-cols-4 gap-2">
        {SEVERITY_ORDER.map((sev) => (
          <div
            key={sev}
            className="bg-gray-800 border border-gray-700 rounded-xl p-3 text-center"
          >
            <p className={clsx('text-xl font-bold', severityColors[sev])}>
              {countBySeverity[sev]}
            </p>
            <p className="text-xs text-gray-500 capitalize mt-0.5">{sev}</p>
          </div>
        ))}
      </div>

      {/* Resolution rate */}
      {alerts.length > 0 && (
        <div className="bg-gray-800 border border-gray-700 rounded-xl p-4">
          <div className="flex justify-between text-sm mb-2">
            <span className="text-gray-400">Resolution rate</span>
            <span className="text-white font-medium">
              {Math.round((resolved.length / alerts.length) * 100)}%
            </span>
          </div>
          <div className="w-full h-2 bg-gray-700 rounded-full overflow-hidden">
            <div
              className="h-full bg-green-500 rounded-full transition-all"
              style={{ width: `${(resolved.length / alerts.length) * 100}%` }}
            />
          </div>
          <div className="flex justify-between text-xs text-gray-500 mt-1">
            <span>{resolved.length} resolved</span>
            <span>{active.length} active</span>
          </div>
        </div>
      )}

      {/* Active alerts list */}
      {active.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          <p className="text-2xl mb-2">✓</p>
          <p className="text-sm font-medium text-gray-400">No active fraud alerts</p>
          <p className="text-xs mt-1">All clear! Fraud prevention is working.</p>
        </div>
      ) : (
        <div className="space-y-2">
          <p className="text-xs font-medium text-gray-400 uppercase tracking-wide">
            Active Alerts ({active.length})
          </p>
          {[...active]
            .sort((a, b) => SEVERITY_ORDER.indexOf(a.severity) - SEVERITY_ORDER.indexOf(b.severity))
            .map((alert) => (
              <FraudAlert
                key={alert.id}
                alert={alert}
                onResolve={onResolve}
                onDismiss={onDismiss}
              />
            ))}
        </div>
      )}
    </div>
  );
}
