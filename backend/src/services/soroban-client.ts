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
import { stellarConfig } from "../config/stellar";

const SIMULATION_ACCOUNT =
  "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN";

export function getServer(): rpc.Server {
  return new rpc.Server(stellarConfig.sorobanRpcUrl, { allowHttp: false });
}

/**
 * Execute a read-only Soroban contract call via simulation
 */
export async function callReadOnly(
  contractId: string,
  method: string,
  args: xdr.ScVal[] = [],
): Promise<any> {
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
