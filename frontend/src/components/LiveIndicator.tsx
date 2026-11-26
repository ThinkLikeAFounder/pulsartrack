'use client';

interface LiveIndicatorProps {
  isConnected: boolean;
}

/** Small badge showing whether real-time WebSocket updates are active. */
export function LiveIndicator({ isConnected }: LiveIndicatorProps) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 text-xs font-medium ${
        isConnected ? 'text-green-500' : 'text-gray-400'
      }`}
      title={isConnected ? 'Receiving live updates' : 'Live updates unavailable; data refreshes periodically'}
    >
      <span
        className={`w-2 h-2 rounded-full ${isConnected ? 'bg-green-500 animate-pulse' : 'bg-gray-400'}`}
        aria-hidden="true"
      />
      {isConnected ? 'Live' : 'Offline'}
    </span>
  );
}
