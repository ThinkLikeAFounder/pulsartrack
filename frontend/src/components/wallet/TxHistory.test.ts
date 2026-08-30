import { describe, it, expect } from 'vitest';
import { normalizeTimestampToMs } from './TxHistory';

describe('normalizeTimestampToMs', () => {
  it('treats values >= 1e12 as already milliseconds and returns them unchanged', () => {
    const ms = 1_711_670_400_000; // 2024-03-28 in ms
    expect(normalizeTimestampToMs(ms)).toBe(ms);
  });

  it('converts seconds (< 1e12) to milliseconds', () => {
    const secs = 1_711_670_400; // 2024-03-28 in Unix seconds
    expect(normalizeTimestampToMs(secs)).toBe(1_711_670_400_000);
  });

  it('never returns a 1970 date for a valid Stellar timestamp in seconds', () => {
    const secs = 1_711_670_400;
    const date = new Date(normalizeTimestampToMs(secs));
    expect(date.getFullYear()).toBeGreaterThan(2000);
  });

  it('handles a timestamp at the boundary (exactly 1e12) as milliseconds', () => {
    // 1e12 ms = year 2001; treat as ms (no conversion)
    expect(normalizeTimestampToMs(1e12)).toBe(1e12);
  });

  it('handles a timestamp just below 1e12 as seconds', () => {
    const justBelowBoundary = 999_999_999_999; // < 1e12 → seconds
    expect(normalizeTimestampToMs(justBelowBoundary)).toBe(999_999_999_999_000);
  });

  it('round-trip: a recent Stellar seconds timestamp maps to a sensible ms date', () => {
    const now = Math.floor(Date.now() / 1000); // current time in seconds
    const ms = normalizeTimestampToMs(now);
    const diff = Math.abs(Date.now() - ms);
    // Should be within 1 second of actual now
    expect(diff).toBeLessThan(1000);
  });

  it('normalises future timestamp in seconds correctly', () => {
    // 1 year from a known epoch second
    const futureSeconds = 1_800_000_000; // year ~2027
    expect(normalizeTimestampToMs(futureSeconds)).toBe(1_800_000_000_000);
  });
});
