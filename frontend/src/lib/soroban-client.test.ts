import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';

vi.mock('./wallet', () => ({
  signTx: vi.fn(),
}));

import { signTx } from './wallet';
import { useTransactionStore } from '../store/tx-store';
// Real (unmocked) SDK module — used to build genuine xdr.ScVal fixtures.
const origSdkModule = await vi.importActual<typeof import('@stellar/stellar-sdk')>(
  '@stellar/stellar-sdk',
);

import {
  callReadOnly,
  callContract,
  getSorobanServer,
  stringToScVal,
  u64ToScVal,
  i128ToScVal,
  u32ToScVal,
  boolToScVal,
  addressToScVal,
} from './soroban-client';

const SIM_ACCOUNT = 'GDWUSKGGFDI4FRXK5EBTRECZSVQSSWJHHJOGH6JWG3AUMFFMQ435DIAG';
const SOROBAN_RPC_URL = 'https://soroban-testnet.stellar.org';
const NETWORK_PASSPHRASE = 'Test SDF Network ; September 2015';

vi.mock('./stellar-config', () => ({
  getSorobanRpcUrl: () => SOROBAN_RPC_URL,
  getNetworkPassphrase: () => NETWORK_PASSPHRASE,
  CONTRACT_IDS: {
    CAMPAIGN_ORCHESTRATOR: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
  },
}));

describe('soroban-client', () => {
  let rpcMock: {
    getAccount: ReturnType<typeof vi.fn>;
    simulateTransaction: ReturnType<typeof vi.fn>;
    sendTransaction: ReturnType<typeof vi.fn>;
    getTransaction: ReturnType<typeof vi.fn>;
  };

  // Records the args passed to `new rpc.Server(...)`.
  let serverCtorSpy: ReturnType<typeof vi.fn<(...ctorArgs: unknown[]) => void>>;

  beforeEach(async () => {
    process.env.NEXT_PUBLIC_SIMULATION_ACCOUNT = SIM_ACCOUNT;
    vi.useFakeTimers();
    vi.clearAllMocks();

    // Reset Zustand tx store between tests (keep _hydrated true)
    useTransactionStore.setState({ transactions: [], _hydrated: true });

    // Construct fresh rpc mock per test and wire it into the mocked @stellar/stellar-sdk
    const sdk = await import('@stellar/stellar-sdk');

    rpcMock = {
      getAccount: vi.fn(),
      simulateTransaction: vi.fn(),
      sendTransaction: vi.fn(),
      getTransaction: vi.fn(),
    };

    // The setup.ts already vi.mock()s @stellar/stellar-sdk. Each call to
    // rpc.Server returns the same object across the board. We simply retarget
    // the implementation here so getAccount / simulateTransaction etc. call
    // our per-test rpcMock functions so we can assert on them.
    // NOTE: vi.clearAllMocks() above (and setup.ts's afterEach) strips any
    // previously installed implementation from rpc.Server, which makes
    // `new rpc.Server(...)` return undefined ("is not a constructor").
    // Reinstall it here, after the clear, on every test.
    // `vi.fn().mockImplementation(fn)` makes the mock itself non-constructable,
    // so `new rpc.Server(...)` in the source would throw. Instead install a real
    // constructable class directly onto the mocked module namespace, and record
    // constructor args on a separate spy for assertions.
    serverCtorSpy = vi.fn();

    class MockRpcServer {
      getAccount = rpcMock.getAccount;
      simulateTransaction = rpcMock.simulateTransaction;
      sendTransaction = rpcMock.sendTransaction;
      getTransaction = rpcMock.getTransaction;

      constructor(...ctorArgs: unknown[]) {
        serverCtorSpy(...ctorArgs);
      }
    }

    (sdk.rpc as unknown as { Server: unknown }).Server = MockRpcServer;
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('helper ScVal builders', () => {
    it('stringToScVal returns a lazy string descriptor', () => {
      expect(stringToScVal('hello')).toEqual({
        __type: 'string',
        value: 'hello',
      });
    });

    it('u64ToScVal returns a lazy u64 descriptor (bigint)', () => {
      expect(u64ToScVal(42)).toEqual({ __type: 'u64', value: BigInt(42) });
      expect(u64ToScVal(BigInt(99))).toEqual({ __type: 'u64', value: BigInt(99) });
    });

    it('i128ToScVal returns a lazy i128 descriptor', () => {
      expect(i128ToScVal(-7)).toEqual({ __type: 'i128', value: BigInt(-7) });
    });

    it('u32ToScVal returns a lazy u32 descriptor', () => {
      expect(u32ToScVal(10)).toEqual({ __type: 'u32', value: 10 });
    });

    it('boolToScVal returns a lazy bool descriptor', () => {
      expect(boolToScVal(true)).toEqual({ __type: 'bool', value: true });
      expect(boolToScVal(false)).toEqual({ __type: 'bool', value: false });
    });

    it('addressToScVal returns a lazy address descriptor', () => {
      expect(addressToScVal('GDWUSKGGFDI4FRXK5EBTRECZSVQSSWJHHJOGH6JWG3AUMFFMQ435DIAG')).toEqual({ __type: 'address', value: 'GDWUSKGGFDI4FRXK5EBTRECZSVQSSWJHHJOGH6JWG3AUMFFMQ435DIAG' });
    });
  });

  describe('getSorobanServer', () => {
    it('creates an rpc.Server with the configured RPC URL', async () => {
      const server = await getSorobanServer();
      expect(serverCtorSpy).toHaveBeenCalledWith(SOROBAN_RPC_URL, { allowHttp: false });
      expect(server).toBeDefined();
    });
  });

  describe('callReadOnly', () => {
    it('successfully performs a read-only simulation and returns the native result', async () => {
      const sdk = await import('@stellar/stellar-sdk');

      rpcMock.getAccount.mockResolvedValue({
        accountId: () => SIM_ACCOUNT,
        sequenceNumber: () => '100',
        incrementSequenceNumber: vi.fn(),
      });

      const mockNativeResult = { name: 'Campaign A', budget: '100' };
      const retvalSymbol = origSdkModule.xdr.ScVal.scvU32(4242);

      const simSuccess = {
        transactionData: { build: vi.fn() },
        minResourceFee: '100',
        result: { retval: retvalSymbol },
      };

      vi.mocked(sdk.rpc.Api.isSimulationError).mockReturnValue(false);
      vi.mocked(sdk.rpc.Api.isSimulationSuccess).mockReturnValue(true);
      rpcMock.simulateTransaction.mockResolvedValue(simSuccess);

      // Make scValToNative transform the retval symbol into our mock native result
      const origSdk = await vi.importActual<typeof import('@stellar/stellar-sdk')>('@stellar/stellar-sdk');
      const scValToNativeSpy = vi
        .spyOn(sdk, 'scValToNative')
        .mockImplementation((val) => {
          if (val === retvalSymbol) return mockNativeResult;
          return origSdk.scValToNative(val);
        });

      const result = await callReadOnly({
        contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
        method: 'get_campaign',
        args: [u32ToScVal(1)],
      });

      expect(rpcMock.getAccount).toHaveBeenCalledWith(SIM_ACCOUNT);
      expect(rpcMock.simulateTransaction).toHaveBeenCalled();
      expect(result).toEqual(mockNativeResult);

      scValToNativeSpy.mockRestore();
    });

    it('returns null when simulation succeeds but retval is missing', async () => {
      const sdk = await import('@stellar/stellar-sdk');

      rpcMock.getAccount.mockResolvedValue({
        accountId: () => SIM_ACCOUNT,
        sequenceNumber: () => '100',
        incrementSequenceNumber: vi.fn(),
      });

      const simSuccessNoRetval = {
        transactionData: { build: vi.fn() },
        minResourceFee: '100',
        result: { retval: undefined },
      };

      vi.mocked(sdk.rpc.Api.isSimulationError).mockReturnValue(false);
      vi.mocked(sdk.rpc.Api.isSimulationSuccess).mockReturnValue(true);
      rpcMock.simulateTransaction.mockResolvedValue(simSuccessNoRetval);

      const result = await callReadOnly({
        contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
        method: 'noop',
      });

      expect(result).toBeNull();
    });

    it('throws when getAccount fails (no simulation account available)', async () => {
      rpcMock.getAccount.mockResolvedValue(null);

      await expect(
        callReadOnly({
          contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
          method: 'get_campaign',
        }),
      ).rejects.toThrow('Could not fetch account for read simulation');
    });

    it('throws when RPC returns simulation error', async () => {
      const sdk = await import('@stellar/stellar-sdk');

      rpcMock.getAccount.mockResolvedValue({
        accountId: () => SIM_ACCOUNT,
        sequenceNumber: () => '100',
        incrementSequenceNumber: vi.fn(),
      });

      const simError = {
        error: 'HostValueError: Invalid input',
      };

      vi.mocked(sdk.rpc.Api.isSimulationError).mockReturnValue(true);
      rpcMock.simulateTransaction.mockResolvedValue(simError);

      await expect(
        callReadOnly({
          contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
          method: 'get_campaign',
        }),
      ).rejects.toThrow('Simulation error: HostValueError: Invalid input');
    });

    it('throws when simulation is not a success and not an error', async () => {
      const sdk = await import('@stellar/stellar-sdk');

      rpcMock.getAccount.mockResolvedValue({
        accountId: () => SIM_ACCOUNT,
        sequenceNumber: () => '100',
        incrementSequenceNumber: vi.fn(),
      });

      vi.mocked(sdk.rpc.Api.isSimulationError).mockReturnValue(false);
      vi.mocked(sdk.rpc.Api.isSimulationSuccess).mockReturnValue(false);
      rpcMock.simulateTransaction.mockResolvedValue({});

      await expect(
        callReadOnly({
          contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
          method: 'get_campaign',
        }),
      ).rejects.toThrow('Simulation failed with no result');
    });
  });

  describe('callContract', () => {
    const SOURCE = 'GDWUSKGGFDI4FRXK5EBTRECZSVQSSWJHHJOGH6JWG3AUMFFMQ435DIAG';
    const TX_HASH = 'txhashabc123';
    // A genuinely well-formed signed envelope: callContract feeds the signed XDR
    // back through TransactionBuilder.fromXDR, which rejects malformed strings.
    const SIGNED_XDR =
      'AAAAAgAAAADtSSjGKNHCxurpAziQWZVhKVknOlxj+TY2wUYUrIc30QAAAGQAAAAAAAAAZQAAAAEAAAAAAAAAAAAAAABqj7AvAAAAAAAAAAEAAAAAAAAAAQAAAADtSSjGKNHCxurpAziQWZVhKVknOlxj+TY2wUYUrIc30QAAAAAAAAAAAJiWgAAAAAAAAAABrIc30QAAAEC+fsH/ri0Ns69asd6tQKpktNZOjnrgnoUehqSr7jSTxKnw609KQKMDdydFxeUUSatXXINOn3DefdBsx3Ywt/0H';

    beforeEach(() => {
      vi.mocked(signTx).mockResolvedValue(SIGNED_XDR);
    });

    it('successful end-to-end contract call with on-chain confirmation', async () => {
      const sdk = await import('@stellar/stellar-sdk');

      rpcMock.getAccount.mockResolvedValue({
        accountId: () => SOURCE,
        sequenceNumber: () => '100',
        incrementSequenceNumber: vi.fn(),
      });

      const simSuccess = {
        transactionData: { build: vi.fn() },
        minResourceFee: '100',
        result: { retval: 'some-scval' },
      };
      vi.mocked(sdk.rpc.Api.isSimulationError).mockReturnValue(false);
      rpcMock.simulateTransaction.mockResolvedValue(simSuccess);

      // Fake assembleTransaction — just produce something that toXDR()
      // returns a fixed string so we can assert signTx was called with it.
      const assembledTx = { toXDR: vi.fn().mockReturnValue('PREPARED_XDR') };
      vi.spyOn(sdk.rpc, 'assembleTransaction').mockReturnValue({
        build: () => assembledTx,
      } as unknown as ReturnType<typeof sdk.rpc.assembleTransaction>);

      // submitResult -> PENDING
      rpcMock.sendTransaction.mockResolvedValue({ status: 'PENDING', hash: TX_HASH });

      // First getTransaction -> NOT_FOUND, second -> SUCCESS
      const retvalScVal = origSdkModule.xdr.ScVal.scvU32(9191);
      const finalResult = { id: 7, name: 'Created!' };
      rpcMock.getTransaction
        .mockResolvedValueOnce({ status: 'NOT_FOUND' })
        .mockResolvedValueOnce({
          status: sdk.rpc.Api.GetTransactionStatus.SUCCESS,
          returnValue: retvalScVal,
        });

      const origSdk = await vi.importActual<typeof import('@stellar/stellar-sdk')>('@stellar/stellar-sdk');
      const scValSpy = vi.spyOn(sdk, 'scValToNative').mockImplementation((v) => {
        if (v === retvalScVal) return finalResult;
        return origSdk.scValToNative(v);
      });

      // Run the call. We must advance timers for the polling loop sleeps.
      const resultPromise = callContract({
        contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
        method: 'create_campaign',
        args: [stringToScVal('New Campaign')],
        source: SOURCE,
        txType: 'campaign_create',
        description: 'Create campaign',
      });

      // Drain setTimeout for each polling iteration: 0 -> delay 2000ms, 1 -> delay 3000ms
      // First sleep after NOT_FOUND
      await vi.runAllTimersAsync();
      // Give Vitest a microtask tick so getTransaction resolves and the loop iterates
      await Promise.resolve();
      // Now the SUCCESS branch is hit; polling returns.
      await vi.runAllTimersAsync();

      const result = await resultPromise;

      expect(rpcMock.getAccount).toHaveBeenCalledWith(SOURCE);
      expect(rpcMock.simulateTransaction).toHaveBeenCalled();
      expect(signTx).toHaveBeenCalledWith('PREPARED_XDR');
      expect(rpcMock.sendTransaction).toHaveBeenCalled();

      expect(result.success).toBe(true);
      expect(result.txHash).toBe(TX_HASH);
      expect(result.result).toEqual(finalResult);

      const tx = useTransactionStore.getState().getTransaction(TX_HASH);
      expect(tx).toBeDefined();
      expect(tx?.status).toBe('success');
      expect(tx?.result).toEqual(finalResult);
      expect(tx?.type).toBe('campaign_create');
      expect(tx?.description).toBe('Create campaign');

      scValSpy.mockRestore();
    });

    it('returns failure when simulation returns error', async () => {
      const sdk = await import('@stellar/stellar-sdk');

      rpcMock.getAccount.mockResolvedValue({
        accountId: () => SOURCE,
        sequenceNumber: () => '100',
        incrementSequenceNumber: vi.fn(),
      });

      vi.mocked(sdk.rpc.Api.isSimulationError).mockReturnValue(true);
      rpcMock.simulateTransaction.mockResolvedValue({
        error: 'InvalidContractInput: bad args',
      });

      const result = await callContract({
        contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
        method: 'create_campaign',
        args: [],
        source: SOURCE,
      });

      expect(result.success).toBe(false);
      expect(result.error).toBe('Simulation failed: InvalidContractInput: bad args');
      expect(signTx).not.toHaveBeenCalled();
    });

    it('returns failure when sendTransaction returns ERROR status', async () => {
      const sdk = await import('@stellar/stellar-sdk');

      rpcMock.getAccount.mockResolvedValue({
        accountId: () => SOURCE,
        sequenceNumber: () => '100',
        incrementSequenceNumber: vi.fn(),
      });

      vi.mocked(sdk.rpc.Api.isSimulationError).mockReturnValue(false);
      rpcMock.simulateTransaction.mockResolvedValue({
        transactionData: { build: vi.fn() },
        minResourceFee: '100',
      });

      vi.spyOn(sdk.rpc, 'assembleTransaction').mockReturnValue({
        build: () => ({ toXDR: () => 'PREPARED_XDR' }),
      } as unknown as ReturnType<typeof sdk.rpc.assembleTransaction>);

      rpcMock.sendTransaction.mockResolvedValue({ status: 'ERROR', hash: undefined });

      const result = await callContract({
        contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
        method: 'create_campaign',
        args: [],
        source: SOURCE,
      });

      expect(result.success).toBe(false);
      expect(result.error).toBe('Transaction submission failed');
    });

    it('returns failure when transaction is confirmed FAILED on-chain', async () => {
      const sdk = await import('@stellar/stellar-sdk');

      rpcMock.getAccount.mockResolvedValue({
        accountId: () => SOURCE,
        sequenceNumber: () => '100',
        incrementSequenceNumber: vi.fn(),
      });

      vi.mocked(sdk.rpc.Api.isSimulationError).mockReturnValue(false);
      rpcMock.simulateTransaction.mockResolvedValue({
        transactionData: { build: vi.fn() },
        minResourceFee: '100',
      });

      vi.spyOn(sdk.rpc, 'assembleTransaction').mockReturnValue({
        build: () => ({ toXDR: () => 'PREPARED_XDR' }),
      } as unknown as ReturnType<typeof sdk.rpc.assembleTransaction>);

      rpcMock.sendTransaction.mockResolvedValue({ status: 'PENDING', hash: TX_HASH });

      rpcMock.getTransaction.mockResolvedValue({
        status: sdk.rpc.Api.GetTransactionStatus.FAILED,
      });

      const resultPromise = callContract({
        contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
        method: 'create_campaign',
        args: [],
        source: SOURCE,
        txType: 'campaign_create',
      });

      await vi.runAllTimersAsync();
      await Promise.resolve();
      await vi.runAllTimersAsync();

      const result = await resultPromise;

      expect(result.success).toBe(false);
      expect(result.error).toBe('Transaction failed on-chain');
      expect(result.txHash).toBe(TX_HASH);

      const tx = useTransactionStore.getState().getTransaction(TX_HASH);
      expect(tx?.status).toBe('failed');
    });

    it('times out after 30 polling iterations without a terminal status', async () => {
      const sdk = await import('@stellar/stellar-sdk');

      rpcMock.getAccount.mockResolvedValue({
        accountId: () => SOURCE,
        sequenceNumber: () => '100',
        incrementSequenceNumber: vi.fn(),
      });

      vi.mocked(sdk.rpc.Api.isSimulationError).mockReturnValue(false);
      rpcMock.simulateTransaction.mockResolvedValue({
        transactionData: { build: vi.fn() },
        minResourceFee: '100',
      });

      vi.spyOn(sdk.rpc, 'assembleTransaction').mockReturnValue({
        build: () => ({ toXDR: () => 'PREPARED_XDR' }),
      } as unknown as ReturnType<typeof sdk.rpc.assembleTransaction>);

      rpcMock.sendTransaction.mockResolvedValue({ status: 'PENDING', hash: TX_HASH });
      rpcMock.getTransaction.mockResolvedValue({ status: 'NOT_FOUND' });

      const resultPromise = callContract({
        contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
        method: 'create_campaign',
        args: [],
        source: SOURCE,
      });

      // Drain 30 polling sleeps
      for (let i = 0; i < 30; i++) {
        await vi.runAllTimersAsync();
        await Promise.resolve();
      }
      // Final timeout-branch timers
      await vi.runAllTimersAsync();

      const result = await resultPromise;

      expect(result.success).toBe(false);
      expect(result.error).toBe('Transaction polling timeout');
      expect(result.txHash).toBe(TX_HASH);

      const tx = useTransactionStore.getState().getTransaction(TX_HASH);
      expect(tx?.status).toBe('timeout');
    });

    it('wraps unexpected thrown errors as success:false with message', async () => {
      rpcMock.getAccount.mockRejectedValue(new Error('Network read ECONNRESET'));

      const result = await callContract({
        contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
        method: 'create_campaign',
        args: [],
        source: SOURCE,
      });

      expect(result.success).toBe(false);
      expect(result.error).toBe('Network read ECONNRESET');
    });

    it('handles non-Error thrown values with Unknown error fallback', async () => {
      rpcMock.getAccount.mockRejectedValue({ weird: 'shape' });

      const result = await callContract({
        contractId: 'CADQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQOBYHA4DQP5KR',
        method: 'create_campaign',
        args: [],
        source: SOURCE,
      });

      expect(result.success).toBe(false);
      expect(result.error).toBe('Unknown error');
    });
  });
});
