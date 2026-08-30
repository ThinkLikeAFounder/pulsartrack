import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useAnalyticsTimeseries } from './useAnalytics';

describe('useAnalyticsTimeseries', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('passes "7d" timeframe to the API URL', async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    } as Response);

    renderHook(() => useAnalyticsTimeseries({ campaignIds: ['1'], timeframe: '7d' }));

    await waitFor(() => expect(fetch).toHaveBeenCalled());

    const url = (fetch as ReturnType<typeof vi.fn>).mock.calls[0][0] as string;
    expect(url).toContain('timeframe=7d');
    expect(url).not.toContain('timeframe=30d');
  });

  it('passes "30d" timeframe to the API URL', async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    } as Response);

    renderHook(() => useAnalyticsTimeseries({ campaignIds: ['1'], timeframe: '30d' }));

    await waitFor(() => expect(fetch).toHaveBeenCalled());

    const url = (fetch as ReturnType<typeof vi.fn>).mock.calls[0][0] as string;
    expect(url).toContain('timeframe=30d');
    expect(url).not.toContain('timeframe=7d');
    expect(url).not.toContain('timeframe=90d');
  });

  it('passes "90d" timeframe to the API URL — not silently replaced with "30d"', async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    } as Response);

    renderHook(() => useAnalyticsTimeseries({ campaignIds: ['1', '2'], timeframe: '90d' }));

    await waitFor(() => expect(fetch).toHaveBeenCalled());

    const url = (fetch as ReturnType<typeof vi.fn>).mock.calls[0][0] as string;
    expect(url).toContain('timeframe=90d');
    expect(url).not.toContain('timeframe=30d');
  });

  it('includes all campaignIds in the query param', async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => [],
    } as Response);

    renderHook(() =>
      useAnalyticsTimeseries({ campaignIds: ['10', '20', '30'], timeframe: '90d' })
    );

    await waitFor(() => expect(fetch).toHaveBeenCalled());

    const url = (fetch as ReturnType<typeof vi.fn>).mock.calls[0][0] as string;
    expect(url).toContain('campaignIds=10,20,30');
  });

  it('sets error state on a non-OK response', async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: false,
      json: async () => null,
    } as Response);

    const { result } = renderHook(() =>
      useAnalyticsTimeseries({ campaignIds: ['1'], timeframe: '30d' })
    );

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.error).toBe('Failed to fetch analytics timeseries');
    expect(result.current.data).toEqual([]);
  });
});
