import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useTransactionStore, type Transaction } from './tx-store';

const baseTx: Omit<Transaction, 'timestamp'> = {
  txHash: 'hash-1',
  type: 'campaign_create',
  status: 'pending',
  description: 'Create campaign',
};

function reset() {
  useTransactionStore.setState({ transactions: [] });
}

describe('tx-store', () => {
  beforeEach(reset);

  describe('addTransaction', () => {
    it('prepends the transaction and stamps a timestamp', () => {
      const before = Date.now();
      useTransactionStore.getState().addTransaction(baseTx);
      const after = Date.now();

      const [tx] = useTransactionStore.getState().transactions;
      expect(tx.txHash).toBe('hash-1');
      expect(tx.timestamp).toBeGreaterThanOrEqual(before);
      expect(tx.timestamp).toBeLessThanOrEqual(after);
    });

    it('keeps newest transactions first', () => {
      const { addTransaction } = useTransactionStore.getState();
      addTransaction({ ...baseTx, txHash: 'old' });
      addTransaction({ ...baseTx, txHash: 'new' });

      expect(
        useTransactionStore.getState().transactions.map((t) => t.txHash),
      ).toEqual(['new', 'old']);
    });
  });

  describe('updateTransaction', () => {
    it('merges updates into the matching transaction only', () => {
      const { addTransaction, updateTransaction } = useTransactionStore.getState();
      addTransaction({ ...baseTx, txHash: 'a' });
      addTransaction({ ...baseTx, txHash: 'b' });

      updateTransaction('a', { status: 'success', result: { ok: true } });

      const txA = useTransactionStore.getState().getTransaction('a');
      const txB = useTransactionStore.getState().getTransaction('b');
      expect(txA?.status).toBe('success');
      expect(txA?.result).toEqual({ ok: true });
      expect(txB?.status).toBe('pending');
    });

    it('is a no-op when the hash is unknown', () => {
      useTransactionStore.getState().addTransaction(baseTx);
      useTransactionStore.getState().updateTransaction('missing', { status: 'failed' });

      expect(useTransactionStore.getState().transactions).toHaveLength(1);
      expect(useTransactionStore.getState().transactions[0].status).toBe('pending');
    });
  });

  describe('queries', () => {
    it('getTransaction returns undefined when not found', () => {
      expect(useTransactionStore.getState().getTransaction('nope')).toBeUndefined();
    });

    it('getPendingTransactions returns only pending ones', () => {
      const { addTransaction, updateTransaction } = useTransactionStore.getState();
      addTransaction({ ...baseTx, txHash: 'a' });
      addTransaction({ ...baseTx, txHash: 'b' });
      addTransaction({ ...baseTx, txHash: 'c' });
      updateTransaction('b', { status: 'success' });

      expect(
        useTransactionStore.getState().getPendingTransactions().map((t) => t.txHash),
      ).toEqual(['c', 'a']);
    });
  });

  describe('clearOldTransactions', () => {
    it('drops settled transactions older than the cutoff but keeps recent and pending ones', () => {
      const now = 1_000_000_000_000;
      vi.spyOn(Date, 'now').mockReturnValue(now);

      const day = 24 * 60 * 60 * 1000;
      useTransactionStore.setState({
        transactions: [
          { ...baseTx, txHash: 'recent-success', status: 'success', timestamp: now - day },
          { ...baseTx, txHash: 'old-success', status: 'success', timestamp: now - 40 * day },
          { ...baseTx, txHash: 'old-pending', status: 'pending', timestamp: now - 40 * day },
        ],
      });

      useTransactionStore.getState().clearOldTransactions(30);

      expect(
        useTransactionStore.getState().transactions.map((t) => t.txHash).sort(),
      ).toEqual(['old-pending', 'recent-success']);

      vi.restoreAllMocks();
    });

    it('defaults the window to 30 days', () => {
      const now = 1_000_000_000_000;
      vi.spyOn(Date, 'now').mockReturnValue(now);
      const day = 24 * 60 * 60 * 1000;

      useTransactionStore.setState({
        transactions: [
          { ...baseTx, txHash: 'at-29d', status: 'failed', timestamp: now - 29 * day },
          { ...baseTx, txHash: 'at-31d', status: 'failed', timestamp: now - 31 * day },
        ],
      });

      useTransactionStore.getState().clearOldTransactions();

      expect(useTransactionStore.getState().transactions.map((t) => t.txHash)).toEqual([
        'at-29d',
      ]);

      vi.restoreAllMocks();
    });
  });
});
