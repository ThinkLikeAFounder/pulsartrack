import { describe, it, expect } from 'vitest';
import { bigintToString, stroopsToXlm } from './bigint';

describe('bigintToString', () => {
  it('preserves values above Number.MAX_SAFE_INTEGER', () => {
    expect(bigintToString('9007199254740993')).toBe('9007199254740993');
    expect(bigintToString(BigInt('9223372036854775807'))).toBe('9223372036854775807');
  });

  it('treats null/undefined as zero', () => {
    expect(bigintToString(null)).toBe('0');
    expect(bigintToString(undefined)).toBe('0');
  });
});

describe('stroopsToXlm', () => {
  it('converts small values', () => {
    expect(stroopsToXlm('100000000')).toBe(10);
    expect(stroopsToXlm(BigInt(15000000))).toBe(1.5);
    expect(stroopsToXlm(0)).toBe(0);
    expect(stroopsToXlm(null)).toBe(0);
  });

  it('does not lose the low digits above 2^53 stroops', () => {
    // 2^53 + 1 stroops: Number() would round this to 2^53.
    const xlm = stroopsToXlm('9007199254740993');
    expect(xlm).toBeCloseTo(900719925.4740993, 6);
  });

  it('handles the BIGINT maximum without throwing', () => {
    expect(Number.isFinite(stroopsToXlm('9223372036854775807'))).toBe(true);
  });
});
