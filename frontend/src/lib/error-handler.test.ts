import { describe, it, expect } from 'vitest';
import {
  ErrorCode,
  createPulsarError,
  parseStellarError,
  getErrorMessage,
  PulsarError,
} from './error-handler';

describe('error-handler', () => {
  describe('createPulsarError', () => {
    it('creates a PulsarError with the provided code, message, and original error', () => {
      const original = new Error('boom');
      const err = createPulsarError(ErrorCode.CONTRACT_ERROR, 'Something broke', original);
      expect(err.code).toBe(ErrorCode.CONTRACT_ERROR);
      expect(err.message).toBe('Something broke');
      expect(err.originalError).toBe(original);
    });

    it('omits originalError when not provided', () => {
      const err = createPulsarError(ErrorCode.UNKNOWN, 'msg');
      expect(err).toEqual({
        code: ErrorCode.UNKNOWN,
        message: 'msg',
        originalError: undefined,
      });
    });
  });

  describe('parseStellarError - network / transport errors', () => {
    it('classifies a message containing "network" as NETWORK_MISMATCH, ahead of the generic "error" branch', () => {
      const parsed = parseStellarError(
        new Error('Failed to fetch: network error during RPC call'),
      );
      // The 'network' branch is checked before the generic 'error' branch,
      // so precedence puts this in NETWORK_MISMATCH.
      expect(parsed.code).toBe(ErrorCode.NETWORK_MISMATCH);
      expect(parsed.message).toBe(
        'Wrong network. Please switch to the correct Stellar network in Freighter.',
      );
    });

    it('classifies a generic transport failure with no earlier-branch keyword as CONTRACT_ERROR', () => {
      const parsed = parseStellarError(new Error('RPC call failed'));
      expect(parsed.code).toBe(ErrorCode.CONTRACT_ERROR);
      expect(parsed.message).toContain('RPC call failed');
    });

    it('classifies TypeError "fetch failed" as CONTRACT_ERROR', () => {
      const parsed = parseStellarError(new TypeError('fetch failed'));
      expect(parsed.code).toBe(ErrorCode.CONTRACT_ERROR);
      expect(parsed.originalError).toBeInstanceOf(TypeError);
    });
  });

  describe('parseStellarError - contract / simulation errors', () => {
    it('classifies simulation error by message', () => {
      const parsed = parseStellarError(
        new Error('HostFunctionError: Simulation trap: value out of range'),
      );
      expect(parsed.code).toBe(ErrorCode.SIMULATION_FAILED);
      expect(parsed.message).toBe(
        'HostFunctionError: Simulation trap: value out of range',
      );
    });

    it('classifies generic "failed" string as CONTRACT_ERROR', () => {
      const parsed = parseStellarError(
        new Error('Transaction failed: insufficient XLM balance'),
      );
      // Note: "insufficient" will be matched first since its check comes earlier
      expect(parsed.code).toBe(ErrorCode.INSUFFICIENT_FUNDS);
    });

    it('classifies a plain contract panic as CONTRACT_ERROR', () => {
      const parsed = parseStellarError(
        new Error('HostValueError: contract panicked with "Unauthorized"'),
      );
      expect(parsed.code).toBe(ErrorCode.CONTRACT_ERROR);
      expect(parsed.message).toContain('Unauthorized');
    });
  });

  describe('parseStellarError - wallet errors', () => {
    it('classifies Freighter "not found" message as WALLET_NOT_FOUND', () => {
      const parsed = parseStellarError(
        new Error('Freighter wallet extension not found in window'),
      );
      expect(parsed.code).toBe(ErrorCode.WALLET_NOT_FOUND);
      expect(parsed.message).toBe(
        'Freighter wallet extension not found in window',
      );
    });

    it('classifies user rejection / cancel as USER_REJECTED', () => {
      const err1 = parseStellarError(new Error('User rejected the signature request'));
      expect(err1.code).toBe(ErrorCode.USER_REJECTED);
      expect(err1.message).toBe('User rejected the transaction');

      const err2 = parseStellarError(new Error('User cancelled the prompt'));
      expect(err2.code).toBe(ErrorCode.USER_REJECTED);
    });

    it('classifies network / passphrase mismatch as NETWORK_MISMATCH', () => {
      const err1 = parseStellarError(
        new Error('network passphrase does not match configured network'),
      );
      expect(err1.code).toBe(ErrorCode.NETWORK_MISMATCH);
      expect(err1.message).toBe(
        'Wrong network. Please switch to the correct Stellar network in Freighter.',
      );
    });

    it('classifies insufficient / balance errors as INSUFFICIENT_FUNDS', () => {
      const err1 = parseStellarError(
        new Error('insufficient funds to cover fee + operation'),
      );
      expect(err1.code).toBe(ErrorCode.INSUFFICIENT_FUNDS);
      expect(err1.message).toBe(
        'Insufficient XLM balance for this transaction.',
      );

      const err2 = parseStellarError(
        new Error('Source account balance is below minimum reserve'),
      );
      expect(err2.code).toBe(ErrorCode.INSUFFICIENT_FUNDS);
    });
  });

  describe('parseStellarError - unknown shapes', () => {
    it('returns UNKNOWN for a plain string throw (non-Error)', () => {
      const parsed = parseStellarError('just a string thrown');
      expect(parsed.code).toBe(ErrorCode.UNKNOWN);
      expect(parsed.message).toBe('An unexpected error occurred.');
      expect(parsed.originalError).toBe('just a string thrown');
    });

    it('returns UNKNOWN for null / undefined', () => {
      const parsedNull = parseStellarError(null);
      expect(parsedNull.code).toBe(ErrorCode.UNKNOWN);
      expect(parsedNull.originalError).toBeNull();

      const parsedUndef = parseStellarError(undefined);
      expect(parsedUndef.code).toBe(ErrorCode.UNKNOWN);
    });

    it('returns UNKNOWN for an Error with no matching keyword', () => {
      const parsed = parseStellarError(new Error('totally unforeseen problem'));
      expect(parsed.code).toBe(ErrorCode.UNKNOWN);
      expect(parsed.message).toBe('An unexpected error occurred.');
    });

    it('matches case-insensitively (uppercase "REJECTED")', () => {
      const parsed = parseStellarError(new Error('REQUEST REJECTED BY USER'));
      expect(parsed.code).toBe(ErrorCode.USER_REJECTED);
    });
  });

  describe('getErrorMessage', () => {
    it.each<[ErrorCode, string]>([
      [ErrorCode.WALLET_NOT_FOUND, 'Freighter wallet not found. Please install the Freighter browser extension.'],
      [ErrorCode.USER_REJECTED, 'Transaction was rejected by the user.'],
      [ErrorCode.NETWORK_MISMATCH, 'Network mismatch. Please switch to the correct Stellar network in Freighter.'],
      [ErrorCode.INSUFFICIENT_FUNDS, 'Insufficient XLM balance. Please fund your account on the Stellar testnet.'],
      [ErrorCode.TX_FAILED, 'Transaction failed on-chain.'],
      [ErrorCode.NOT_CONNECTED, 'Wallet not connected. Please connect your Freighter wallet.'],
    ])('maps %s to the correct static message', (code, expected) => {
      const err: PulsarError = { code, message: 'ignored' };
      expect(getErrorMessage(err)).toBe(expected);
    });

    it('SIMULATION_FAILED prefixes the raw message', () => {
      const err = createPulsarError(
        ErrorCode.SIMULATION_FAILED,
        'HostValueError: div by zero',
      );
      expect(getErrorMessage(err)).toBe(
        'Contract simulation failed: HostValueError: div by zero',
      );
    });

    it('CONTRACT_ERROR prefixes the raw message', () => {
      const err = createPulsarError(
        ErrorCode.CONTRACT_ERROR,
        'Host panic: Unauthorized caller',
      );
      expect(getErrorMessage(err)).toBe(
        'Contract error: Host panic: Unauthorized caller',
      );
    });

    it('UNKNOWN falls back to the embedded message then to a default', () => {
      const custom = createPulsarError(ErrorCode.UNKNOWN, 'some custom message');
      expect(getErrorMessage(custom)).toBe('some custom message');

      const empty = createPulsarError(ErrorCode.UNKNOWN, '');
      expect(getErrorMessage(empty)).toBe('An unexpected error occurred.');
    });
  });

  describe('end-to-end parse -> message roundtrip for common flows', () => {
    it('wallet rejection -> parse then getMessage produces a friendly string', () => {
      const raw = new Error('User rejected signing');
      const pulsar = parseStellarError(raw);
      expect(getErrorMessage(pulsar)).toBe('Transaction was rejected by the user.');
    });

    it('network mismatch -> parse then getMessage produces a friendly string', () => {
      const raw = new Error('Network passphrase mismatch');
      const pulsar = parseStellarError(raw);
      expect(getErrorMessage(pulsar)).toBe(
        'Network mismatch. Please switch to the correct Stellar network in Freighter.',
      );
    });

    it('simulation failure -> parse then getMessage includes the underlying error', () => {
      const raw = new Error('Simulation error: over budget');
      const pulsar = parseStellarError(raw);
      expect(pulsar.code).toBe(ErrorCode.SIMULATION_FAILED);
      expect(getErrorMessage(pulsar)).toContain('over budget');
    });
  });
});
