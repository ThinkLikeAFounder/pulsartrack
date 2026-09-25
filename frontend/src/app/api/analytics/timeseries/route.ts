import { NextRequest, NextResponse } from 'next/server';

export interface AnalyticsTimeseriesPoint {
  date: string;
  impressions: number;
  clicks: number;
}

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const campaignIds = searchParams.get('campaignIds') || '';
  const timeframe = searchParams.get('timeframe') || '30d';

  if (!campaignIds) {
    return NextResponse.json(
      { error: 'campaignIds query parameter is required' },
      { status: 400 }
    );
  }

  // TODO(#916): Implement real data source — either query the analytics tables
  // via the backend, call the analytics-aggregator contract, or proxy to the
  // backend's analytics endpoint using NEXT_PUBLIC_API_URL.
  const data: AnalyticsTimeseriesPoint[] = [];

  return NextResponse.json(data);
}
