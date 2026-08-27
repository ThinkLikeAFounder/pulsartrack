import { describe, it, expect, vi, beforeEach } from 'vitest';

// The global test-setup mocks './services/soroban-client' with bare stubs for
// the benefit of other suites. This file tests that module, so the mock is
// removed here — otherwise the functions under test are stubs returning
// undefined and every assertion is vacuous.
vi.unmock('./soroban-client');

import { callReadOnly, toAddressScVal, toU64ScVal, getServer } from './soroban-client';

// A real, well-formed ed25519 public key. Note this is deliberately NOT the
// SIMULATION_ACCOUNT default used elsewhere in the codebase: that string is 55
// characters and fails strkey validation, so it cannot be used to exercise the
// genuine Address conversion path.
const VALID_ACCOUNT = 'GDVEU3DD4KOFECV66VIHWEZOYX4ZKR3WV27L464SIIPOU2IUI3JCZA57';

/**
 * Real SDK primitives, loaded lazily inside tests. `@stellar/stellar-sdk` is
 * mocked below, so the ScVal helpers are asserted against genuine XDR values
 * rather than against the stubs.
 */
async function realSdk() {
  return vi.importActual<typeof import('@stellar/stellar-sdk')>(
    '@stellar/stellar-sdk',
  );
}

// Mock stellar-sdk
const mockSimulateTransaction = vi.fn();
const mockGetAccount = vi.fn();

vi.mock('@stellar/stellar-sdk', async () => {
  // `vi.importActual` is async — it must be awaited. Spreading the unawaited
  // promise silently discarded every real SDK export.
  const actual = await vi.importActual<typeof import('@stellar/stellar-sdk')>(
    '@stellar/stellar-sdk',
  );
  return {
    ...actual,
    // Only the network layer is mocked. `Address`, `nativeToScVal` and `xdr`
    // deliberately stay real so the ScVal conversion helpers are asserted
    // against their genuine output rather than against a stub.
    rpc: {
      ...actual.rpc,
      // A `function`, not an arrow: the implementation calls `new rpc.Server()`
      // and arrow functions cannot be construed as constructors.
      Server: vi.fn().mockImplementation(function () {
        return {
          simulateTransaction: mockSimulateTransaction,
          getAccount: mockGetAccount,
        };
      }),
      Api: {
        isSimulationError: (sim: any) => sim?.error !== undefined && !sim?.result,
        isSimulationSuccess: (sim: any) => sim?.result !== undefined,
      },
    },
    Contract: vi.fn().mockImplementation(function () {
      return { call: vi.fn().mockReturnValue('mock_op') };
    }),
    TransactionBuilder: Object.assign(
      vi.fn().mockImplementation(function () {
        return {
          addOperation: vi.fn().mockReturnThis(),
          setTimeout: vi.fn().mockReturnThis(),
          build: vi.fn().mockReturnValue('mock_tx'),
        };
      }),
      { fromXDR: vi.fn() },
    ),
    BASE_FEE: '100',
    scValToNative: vi.fn().mockReturnValue('decoded_value'),
  };
});

vi.mock('../config/stellar', () => ({
  stellarConfig: {
    sorobanRpcUrl: 'https://soroban-rpc.testnet.stellar.org',
    networkPassphrase: 'Test SDF Network ; September 2024',
  },
  STELLAR_REQUEST_TIMEOUT_MS: 30000,
  getHorizonServer: vi.fn(),
}));

vi.mock('../lib/logger', () => ({
  logger: { info: vi.fn(), error: vi.fn(), warn: vi.fn(), debug: vi.fn() },
}));

beforeEach(() => {
  vi.clearAllMocks();
  mockGetAccount.mockResolvedValue({ accountId: 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN' });
});

describe('soroban-client', () => {
  describe('callReadOnly', () => {
    it('returns decoded value on successful simulation', async () => {
      mockSimulateTransaction.mockResolvedValue({
        result: { retval: 'mock_retval' },
      });

      const result = await callReadOnly('CABC123...', 'get_balance');
      expect(result).toBe('decoded_value');
      expect(mockSimulateTransaction).toHaveBeenCalled();
    });

    it('throws on simulation error', async () => {
      mockSimulateTransaction.mockResolvedValue({ error: 'contract error' });

      await expect(callReadOnly('CABC123...', 'fail')).rejects.toThrow('Simulation error');
    });

    it('throws on missing result', async () => {
      mockSimulateTransaction.mockResolvedValue({});

      await expect(callReadOnly('CABC123...', 'unknown')).rejects.toThrow('no result');
    });

    it('throws on placeholder contract ID', async () => {
      await expect(callReadOnly('PLACEHOLDER', 'method')).rejects.toThrow('missing or a placeholder');
    });

    it('throws on empty contract ID', async () => {
      await expect(callReadOnly('', 'method')).rejects.toThrow('missing or a placeholder');
    });
  });

  describe('toAddressScVal', () => {
    it('creates an address ScVal that round-trips back to the same address', async () => {
      const { xdr, Address, scValToNative } = await realSdk();
      const result = toAddressScVal(VALID_ACCOUNT);

      expect(result).toBeInstanceOf(xdr.ScVal);
      expect(result.switch()).toBe(xdr.ScValType.scvAddress());
      expect(Address.fromScVal(result).toString()).toBe(VALID_ACCOUNT);
      expect(scValToNative(result)).toBe(VALID_ACCOUNT);
    });

    it('rejects a malformed address', () => {
      expect(() => toAddressScVal('not-a-valid-address')).toThrow();
    });
  });

  describe('toU64ScVal', () => {
    it('creates a u64 ScVal that round-trips back to the same value', async () => {
      const { xdr, scValToNative } = await realSdk();
      const result = toU64ScVal(42);

      expect(result).toBeInstanceOf(xdr.ScVal);
      expect(result.switch()).toBe(xdr.ScValType.scvU64());
      expect(result.u64().toString()).toBe('42');
      expect(scValToNative(result)).toBe(BigInt(42));
    });

    it('accepts a bigint and preserves large values beyond Number.MAX_SAFE_INTEGER', async () => {
      const { xdr, scValToNative } = await realSdk();
      const big = BigInt('18446744073709551615'); // u64 max
      const result = toU64ScVal(big);

      expect(result.switch()).toBe(xdr.ScValType.scvU64());
      expect(scValToNative(result)).toBe(big);
    });
  });
});
