import { useEffect, useMemo, useState } from 'react';

export interface AnalyticsTimeseriesPoint {
  date: string;
  impressions: number;
  clicks: number;
}

interface UseAnalyticsTimeseriesOptions {
  campaignIds: string[];
  timeframe: '7d' | '30d' | '90d';
}

export function useAnalyticsTimeseries({ campaignIds, timeframe }: UseAnalyticsTimeseriesOptions) {
  const [data, setData] = useState<AnalyticsTimeseriesPoint[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Memoize campaignIds to prevent unnecessary re-fetches when the array reference changes
  // but the content remains the same.
  const campaignIdsKey = useMemo(() => campaignIds.join(','), [campaignIds]);

  useEffect(() => {
    const controller = new AbortController();

    async function load() {
      setLoading(true);
      setError(null);

      try {
        // Replace with actual API endpoint or contract call
        const res = await fetch(
          `/api/analytics/timeseries?campaignIds=${campaignIdsKey}&timeframe=${timeframe}`,
          { signal: controller.signal }
        );
        if (!res.ok) throw new Error('Failed to fetch analytics timeseries');
        const result: AnalyticsTimeseriesPoint[] = await res.json();
        setData(result);
        setLoading(false);
      } catch (err) {
        if (err instanceof Error && err.name === 'AbortError') {
          return;
        }

        setError(err instanceof Error ? err.message : 'Failed to fetch analytics timeseries');
        setLoading(false);
      }
    }

    load();

    return () => {
      controller.abort();
    };
  }, [campaignIdsKey, timeframe]);

  return { data, loading, error };
}
