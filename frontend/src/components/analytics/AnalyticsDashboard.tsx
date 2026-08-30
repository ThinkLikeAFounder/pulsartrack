"use client";

import { Campaign } from "@/types/contracts";
import { MetricsCard } from "./MetricsCard";
import { ImpressionChart } from "./ImpressionChart";
import { formatXlm, formatNumber } from "@/lib/display-utils";
import { useAnalyticsTimeseries } from "@/hooks/useAnalytics";

interface AnalyticsDashboardProps {
  campaigns: Campaign[];
  timeframe?: "7d" | "30d" | "90d";
  onTimeframeChange?: (tf: "7d" | "30d" | "90d") => void;
}

// ...existing code...

export function AnalyticsDashboard({
  campaigns,
  timeframe = "30d",
  onTimeframeChange,
}: AnalyticsDashboardProps) {
  const totalImpressions = campaigns.reduce((a, c) => a + c.impressions, 0n);
  const totalClicks = campaigns.reduce((a, c) => a + c.clicks, 0n);
  const totalSpent = campaigns.reduce((a, c) => a + c.spent, 0n);
  
  const ctrVal = totalImpressions > 0n ? (Number(totalClicks) / Number(totalImpressions)) * 100 : 0;
  const ctr = Number.isFinite(ctrVal) ? ctrVal : 0;

  const rawCpm =
    totalImpressions > 0n
      ? (Number(totalSpent) / 1e7 / Number(totalImpressions)) * 1000
      : 0;
  const cpm = Number.isFinite(rawCpm) ? rawCpm.toFixed(4) : "0.0000";

  const days = timeframe === "7d" ? 7 : timeframe === "30d" ? 30 : 90;
  const campaignIds = campaigns.map(c => c.campaign_id.toString());
  const { data: timeseries, loading, error } = useAnalyticsTimeseries({ campaignIds, timeframe });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-white font-semibold text-lg">Analytics</h2>
        {onTimeframeChange && (
          <div className="flex gap-1 bg-gray-800 border border-gray-700 rounded-lg p-1">
            {(["7d", "30d", "90d"] as const).map((tf) => (
              <button
                key={tf}
                onClick={() => onTimeframeChange(tf)}
                className={`px-3 py-1 rounded text-xs font-medium transition-colors ${timeframe === tf
                    ? "bg-indigo-600 text-white"
                    : "text-gray-400 hover:text-white"
                  }`}
              >
                {tf}
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <MetricsCard
          label="Impressions"
          value={formatNumber(totalImpressions)}
          subValue="total served"
          variant="blue"
        />
        <MetricsCard
          label="Clicks"
          value={formatNumber(totalClicks)}
          subValue={`${ctr.toFixed(2)}% CTR`}
          variant="green"
        />
        <MetricsCard
          label="Total Spent"
          value={`${formatXlm(totalSpent)} XLM`}
          subValue="across campaigns"
          variant="purple"
        />
        <MetricsCard
          label="CPM"
          value={`${cpm} XLM`}
          subValue="cost per 1k impressions"
          variant="orange"
        />
      </div>

      <div className="bg-gray-800 border border-gray-700 rounded-xl p-4">
        <h3 className="text-sm font-medium text-gray-300 mb-4">
          Impressions & Clicks — Last {days} Days
        </h3>
        {error ? (
          <div className="text-yellow-400 text-xs mb-2">
            Sample data — deploy contracts to see real metrics
          </div>
        ) : null}
        <ImpressionChart data={
          !loading && timeseries && timeseries.length > 0
            ? timeseries.map((point) => ({
              label: point.date,
              impressions: point.impressions,
              clicks: point.clicks,
            }))
            : []
        } height={180} />
      </div>

      {/* Campaign breakdown */}
      {campaigns.length > 0 && (
        <div className="bg-gray-800 border border-gray-700 rounded-xl p-4">
          <h3 className="text-sm font-medium text-gray-300 mb-3">
            Campaign Breakdown
          </h3>
          <table className="w-full text-sm">
            <thead>
              <tr className="text-xs text-gray-500 border-b border-gray-700">
                <th className="text-left pb-2">Campaign</th>
                <th className="text-right pb-2">Impressions</th>
                <th className="text-right pb-2">Clicks</th>
                <th className="text-right pb-2">CTR</th>
                <th className="text-right pb-2">Spent (XLM)</th>
              </tr>
            </thead>
            <tbody>
              {campaigns.map((c) => {
                const campaignCtrVal =
                  c.impressions > 0n
                    ? (
                      (Number(c.clicks) / Number(c.impressions)) *
                      100
                    )
                    : 0;
                const campaignCtr = Number.isFinite(campaignCtrVal) ? campaignCtrVal.toFixed(2) : "0.00";
                return (
                  <tr
                    key={c.campaign_id.toString()}
                    className="border-b border-gray-700/50 hover:bg-gray-700/30"
                  >
                    <td className="py-2 text-white truncate max-w-[160px]">
                      {c.title}
                    </td>
                    <td className="py-2 text-right text-gray-300">
                      {formatNumber(c.impressions)}
                    </td>
                    <td className="py-2 text-right text-gray-300">
                      {formatNumber(c.clicks)}
                    </td>
                    <td className="py-2 text-right text-cyan-400">
                      {campaignCtr}%
                    </td>
                    <td className="py-2 text-right text-gray-300">
                      {formatXlm(c.spent)}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
