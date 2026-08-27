import { describe, it, expect, vi, beforeEach } from 'vitest';
import { getAccountDetails, isAccountFunded, getAccountTransactions } from './horizon';

// `vi.mock` factories are hoisted above module-level consts, so these have to
// be created inside `vi.hoisted` to exist by the time the factory runs.
const { mockLoadAccount, mockTransactions } = vi.hoisted(() => ({
  mockLoadAccount: vi.fn(),
  mockTransactions: vi.fn(),
}));

vi.mock('../config/stellar', () => ({
  getHorizonServer: vi.fn().mockReturnValue({
    loadAccount: mockLoadAccount,
    transactions: mockTransactions,
    httpClient: { defaults: { timeout: 0 }, get: vi.fn() },
    ledgers: vi.fn().mockReturnValue({
      cursor: vi.fn().mockReturnThis(),
      stream: vi.fn().mockReturnValue({ close: vi.fn() }),
    }),
    feeStats: vi.fn().mockResolvedValue({}),
    operations: vi.fn().mockReturnValue({
      forAccount: vi.fn().mockReturnThis(),
      limit: vi.fn().mockReturnThis(),
      order: vi.fn().mockReturnThis(),
      call: vi.fn().mockResolvedValue({ records: [] }),
    }),
  }),
  STELLAR_REQUEST_TIMEOUT_MS: 30000,
}));

vi.mock('../lib/logger', () => ({
  logger: { info: vi.fn(), error: vi.fn(), warn: vi.fn(), debug: vi.fn() },
}));

beforeEach(() => {
  vi.clearAllMocks();
});

describe('horizon', () => {
  describe('getAccountDetails', () => {
    it('returns account details on success', async () => {
      mockLoadAccount.mockResolvedValue({
        sequence: '12345',
        balances: [
          { asset_type: 'native', balance: '100.5' },
          { asset_type: 'credit_alphanum4', balance: '50' },
        ],
      });

      const result = await getAccountDetails('GABC123...');
      expect(result).toEqual({
        address: 'GABC123...',
        sequenceNumber: '12345',
        xlmBalance: 100.5,
        balances: expect.any(Array),
      });
    });

    it('returns null for 404', async () => {
      mockLoadAccount.mockRejectedValue({ response: { status: 404 } });

      const result = await getAccountDetails('GNOTFOUND...');
      expect(result).toBeNull();
    });

    it('throws on other errors', async () => {
      mockLoadAccount.mockRejectedValue(new Error('network error'));

      await expect(getAccountDetails('GABC123...')).rejects.toThrow('network error');
    });
  });

  describe('isAccountFunded', () => {
    it('returns true when account has >= 1 XLM', async () => {
      mockLoadAccount.mockResolvedValue({
        sequence: '1',
        balances: [{ asset_type: 'native', balance: '5.0' }],
      });

      expect(await isAccountFunded('GABC123...')).toBe(true);
    });

    it('returns false when account not found', async () => {
      mockLoadAccount.mockRejectedValue({ response: { status: 404 } });

      expect(await isAccountFunded('GNOTFOUND...')).toBe(false);
    });
  });

  describe('getAccountTransactions', () => {
    it('calls Horizon transactions endpoint', async () => {
      const mockCall = vi.fn().mockResolvedValue({ records: [] });
      mockTransactions.mockReturnValue({
        forAccount: vi.fn().mockReturnValue({
          limit: vi.fn().mockReturnValue({
            order: vi.fn().mockReturnValue({
              call: mockCall,
            }),
          }),
        }),
      });

      const result = await getAccountTransactions('GABC123...', 10);
      expect(result).toEqual({ records: [] });
    });
  });
});
