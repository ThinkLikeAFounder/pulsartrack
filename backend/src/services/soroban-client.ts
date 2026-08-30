import {
  Contract,
  rpc,
  TransactionBuilder,
  BASE_FEE,
  scValToNative,
  nativeToScVal,
  xdr,
  Address,
} from "@stellar/stellar-sdk";
import { STELLAR_REQUEST_TIMEOUT_MS, stellarConfig } from "../config/stellar";

import { logger } from "../lib/logger";

const SIMULATION_ACCOUNT =
  process.env.SIMULATION_ACCOUNT || "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN";

/**
 * Validate that the simulation account exists and is funded on the target network.
 * In production, we fail hard if it doesn't exist.
 */
export async function validateSimulationAccount() {
  const server = getServer();
  try {
    await server.getAccount(SIMULATION_ACCOUNT);
    logger.info(`[Soroban] Simulation account ${SIMULATION_ACCOUNT} validated`);
  } catch (err) {
    logger.error(`[Soroban] Simulation account ${SIMULATION_ACCOUNT} not found or unfunded. Read-only calls will fail.`);
    if (process.env.NODE_ENV === "production") {
      logger.fatal("[Soroban] Aborting due to missing simulation account in production");
      process.exit(1);
    }
  }
}

export function getServer(): rpc.Server {
  return new rpc.Server(stellarConfig.sorobanRpcUrl, {
    allowHttp: false,
    timeout: STELLAR_REQUEST_TIMEOUT_MS,
  });
}

/**
 * Execute a read-only Soroban contract call via simulation
 */
export async function callReadOnly(
  contractId: string,
  method: string,
  args: xdr.ScVal[] = [],
): Promise<any> {
  if (!contractId || contractId === 'PLACEHOLDER') {
    throw new Error(
      `callReadOnly("${method}"): contract ID is missing or a placeholder. ` +
      `Set the corresponding CONTRACT_* environment variable.`,
    );
  }

  const server = getServer();
  const contract = new Contract(contractId);
  const account = await server.getAccount(SIMULATION_ACCOUNT);

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: stellarConfig.networkPassphrase,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  const sim = await server.simulateTransaction(tx);

  if (rpc.Api.isSimulationError(sim)) {
    throw new Error(`Simulation error: ${(sim as any).error}`);
  }

  if (!rpc.Api.isSimulationSuccess(sim)) {
    throw new Error("Simulation returned no result");
  }

  const retval = (sim as rpc.Api.SimulateTransactionSuccessResponse).result
    ?.retval;
  return retval ? scValToNative(retval) : null;
}

export function toAddressScVal(address: string): xdr.ScVal {
  return new Address(address).toScVal();
}

export function toU64ScVal(value: number | bigint): xdr.ScVal {
  return nativeToScVal(BigInt(value), { type: "u64" });
}
