"use client";

import {
  Contract,
  rpc,
  TransactionBuilder,
  BASE_FEE,
  xdr,
  Address,
  nativeToScVal,
  scValToNative,
} from "@stellar/stellar-sdk";
import {
  getSorobanRpcUrl,
  getNetworkPassphrase,
  CONTRACT_IDS,
} from "./stellar-config";
import { signTx } from "./wallet";
import { useTransactionStore, TransactionType } from "../store/tx-store";

export interface ContractCallOptions {
  contractId: string;
  method: string;
  args?: xdr.ScVal[];
  source: string; // Public key of caller
  txType?: TransactionType;
  description?: string;
}

export interface ContractCallResult {
  success: boolean;
  result?: any;
  txHash?: string;
  error?: string;
}

export interface ReadOnlyOptions {
  contractId: string;
  method: string;
  args?: xdr.ScVal[];
}

/**
 * Get Soroban RPC server instance
 */
export function getSorobanServer(): rpc.Server {
  return new rpc.Server(getSorobanRpcUrl(), { allowHttp: false });
}

/**
 * Call a read-only Soroban contract function (simulation only)
 */
export async function callReadOnly(options: ReadOnlyOptions): Promise<any> {
  const server = getSorobanServer();
  const contract = new Contract(options.contractId);

  // The Soroban RPC Server and SDK require a source account to compute the footprint and fee for a simulation,
  // even for read-only invocations.
  let simulationAccount = process.env.NEXT_PUBLIC_SIMULATION_ACCOUNT;

  if (!simulationAccount) {
    if (process.env.NODE_ENV === "development") {
      // Fall back to a well-known placeholder account during development
      simulationAccount =
        "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN";
    } else {
      throw new Error(
        "NEXT_PUBLIC_SIMULATION_ACCOUNT environment variable is not set. A source account is required for contract simulations.",
      );
    }
  }

  const account = await server.getAccount(simulationAccount).catch(() => null);

  if (!account) {
    throw new Error("Could not fetch account for read simulation");
  }

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: getNetworkPassphrase(),
  })
    .addOperation(contract.call(options.method, ...(options.args || [])))
    .setTimeout(30)
    .build();

  const simResult = await server.simulateTransaction(tx);

  if (rpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation error: ${(simResult as any).error}`);
  }

  if (!rpc.Api.isSimulationSuccess(simResult)) {
    throw new Error("Simulation failed with no result");
  }

  const returnVal = (simResult as rpc.Api.SimulateTransactionSuccessResponse)
    .result?.retval;
  if (!returnVal) return null;

  return scValToNative(returnVal);
}

/**
 * Call a Soroban contract function (requires wallet signing)
 */
export async function callContract(
  options: ContractCallOptions,
): Promise<ContractCallResult> {
  const server = getSorobanServer();
  const contract = new Contract(options.contractId);

  try {
    const account = await server.getAccount(options.source);

    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: getNetworkPassphrase(),
    })
      .addOperation(contract.call(options.method, ...(options.args || [])))
      .setTimeout(30)
      .build();

    // Simulate to get footprint and fee estimate
    const simResult = await server.simulateTransaction(tx);

    if (rpc.Api.isSimulationError(simResult)) {
      return {
        success: false,
        error: `Simulation failed: ${(simResult as any).error}`,
      };
    }

    // Assemble transaction with simulation data
    const preparedTx = rpc.assembleTransaction(tx, simResult).build();

    // Sign with Freighter
    const signedXdr = await signTx(preparedTx.toXDR());

    // Submit
    const submitResult = await server.sendTransaction(
      TransactionBuilder.fromXDR(signedXdr, getNetworkPassphrase()) as any,
    );

    if (submitResult.status === "ERROR") {
      return { success: false, error: "Transaction submission failed" };
    }

    const txHash = submitResult.hash;

    // Save transaction to persistent store before polling
    const { addTransaction, updateTransaction } =
      useTransactionStore.getState();
    addTransaction({
      txHash,
      type: options.txType || "other",
      status: "pending",
      description: options.description || `${options.method} on contract`,
    });

    // Poll for result
    for (let i = 0; i < 10; i++) {
      await new Promise((resolve) => setTimeout(resolve, 3000));
      const getResult = await server.getTransaction(txHash);

      if (getResult.status === rpc.Api.GetTransactionStatus.SUCCESS) {
        const returnVal = (
          getResult as rpc.Api.GetSuccessfulTransactionResponse
        ).returnValue;
        const result = returnVal ? scValToNative(returnVal) : null;

        // Update transaction status to success
        updateTransaction(txHash, {
          status: "success",
          result,
        });

        return {
          success: true,
          txHash,
          result,
        };
      }

      if (getResult.status === rpc.Api.GetTransactionStatus.FAILED) {
        // Update transaction status to failed
        updateTransaction(txHash, {
          status: "failed",
          error: "Transaction failed on-chain",
        });

        return { success: false, txHash, error: "Transaction failed on-chain" };
      }
    }

    // Polling timeout - transaction remains pending
    return { success: false, error: "Transaction polling timeout", txHash };
  } catch (err: any) {
    return { success: false, error: err?.message || "Unknown error" };
  }
}

/**
 * Helper: Convert string to ScVal
 */
export function stringToScVal(value: string): xdr.ScVal {
  return nativeToScVal(value, { type: "string" });
}

/**
 * Helper: Convert number to u64 ScVal
 */
export function u64ToScVal(value: number | bigint): xdr.ScVal {
  return nativeToScVal(BigInt(value), { type: "u64" });
}

/**
 * Helper: Convert number to i128 ScVal
 */
export function i128ToScVal(value: number | bigint): xdr.ScVal {
  return nativeToScVal(BigInt(value), { type: "i128" });
}

/**
 * Helper: Convert number to u32 ScVal
 */
export function u32ToScVal(value: number): xdr.ScVal {
  return nativeToScVal(value, { type: "u32" });
}

/**
 * Helper: Convert boolean to ScVal
 */
export function boolToScVal(value: boolean): xdr.ScVal {
  return nativeToScVal(value, { type: "bool" });
}

/**
 * Helper: Convert Stellar address to ScVal
 */
export function addressToScVal(address: string): xdr.ScVal {
  return new Address(address).toScVal();
}
