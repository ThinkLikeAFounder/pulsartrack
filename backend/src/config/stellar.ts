import { Horizon, Networks } from "@stellar/stellar-sdk";
import { logger } from "../lib/logger";
import { loadEnv } from "./env";

const env = loadEnv();
const NETWORK = env.STELLAR_NETWORK;
const HORIZON_URL =
  NETWORK === "mainnet"
    ? "https://horizon.stellar.org"
    : "https://horizon-testnet.stellar.org";

const SOROBAN_RPC_URL =
  NETWORK === "mainnet"
    ? "https://mainnet.sorobanrpc.com"
    : "https://soroban-testnet.stellar.org";

const NETWORK_PASSPHRASE =
  NETWORK === "mainnet" ? Networks.PUBLIC : Networks.TESTNET;

export const STELLAR_REQUEST_TIMEOUT_MS = env.STELLAR_REQUEST_TIMEOUT_MS;

export const stellarConfig = {
  network: NETWORK,
  horizonUrl: HORIZON_URL,
  sorobanRpcUrl: SOROBAN_RPC_URL,
  networkPassphrase: NETWORK_PASSPHRASE,
  requestTimeoutMs: STELLAR_REQUEST_TIMEOUT_MS,
};

export function getHorizonServer(): Horizon.Server {
  return new Horizon.Server(HORIZON_URL);
}

export const CONTRACT_IDS = {
  ACCESS_CONTROL: env.CONTRACT_ACCESS_CONTROL || "",
  AD_REGISTRY: env.CONTRACT_AD_REGISTRY || "",
  ANALYTICS_AGGREGATOR: env.CONTRACT_ANALYTICS_AGGREGATOR || "",
  ANOMALY_DETECTOR: env.CONTRACT_ANOMALY_DETECTOR || "",
  AUCTION_ENGINE: env.CONTRACT_AUCTION_ENGINE || "",
  AUDIENCE_SEGMENTS: env.CONTRACT_AUDIENCE_SEGMENTS || "",
  BUDGET_OPTIMIZER: env.CONTRACT_BUDGET_OPTIMIZER || "",
  CAMPAIGN_ANALYTICS: env.CONTRACT_CAMPAIGN_ANALYTICS || "",
  CAMPAIGN_LIFECYCLE: env.CONTRACT_CAMPAIGN_LIFECYCLE || "",
  CAMPAIGN_ORCHESTRATOR: env.CONTRACT_CAMPAIGN_ORCHESTRATOR || "",
  CREATIVE_MARKETPLACE: env.CONTRACT_CREATIVE_MARKETPLACE || "",
  DISPUTE_RESOLUTION: env.CONTRACT_DISPUTE_RESOLUTION || "",
  ESCROW_VAULT: env.CONTRACT_ESCROW_VAULT || "",
  FRAUD_PREVENTION: env.CONTRACT_FRAUD_PREVENTION || "",
  GOVERNANCE_CORE: env.CONTRACT_GOVERNANCE_CORE || "",
  GOVERNANCE_DAO: env.CONTRACT_GOVERNANCE_DAO || "",
  GOVERNANCE_TOKEN: env.CONTRACT_GOVERNANCE_TOKEN || "",
  IDENTITY_REGISTRY: env.CONTRACT_IDENTITY_REGISTRY || "",
  KYC_REGISTRY: env.CONTRACT_KYC_REGISTRY || "",
  LIQUIDITY_POOL: env.CONTRACT_LIQUIDITY_POOL || "",
  MILESTONE_TRACKER: env.CONTRACT_MILESTONE_TRACKER || "",
  MULTISIG_TREASURY: env.CONTRACT_MULTISIG_TREASURY || "",
  ORACLE_INTEGRATION: env.CONTRACT_ORACLE_INTEGRATION || "",
  PAYMENT_PROCESSOR: env.CONTRACT_PAYMENT_PROCESSOR || "",
  PAYOUT_AUTOMATION: env.CONTRACT_PAYOUT_AUTOMATION || "",
  PERFORMANCE_ORACLE: env.CONTRACT_PERFORMANCE_ORACLE || "",
  PRIVACY_LAYER: env.CONTRACT_PRIVACY_LAYER || "",
  PUBLISHER_NETWORK: env.CONTRACT_PUBLISHER_NETWORK || "",
  PUBLISHER_REPUTATION: env.CONTRACT_PUBLISHER_REPUTATION || "",
  PUBLISHER_VERIFICATION: env.CONTRACT_PUBLISHER_VERIFICATION || "",
  RECURRING_PAYMENT: env.CONTRACT_RECURRING_PAYMENT || "",
  REFUND_PROCESSOR: env.CONTRACT_REFUND_PROCESSOR || "",
  REVENUE_SETTLEMENT: env.CONTRACT_REVENUE_SETTLEMENT || "",
  REWARDS_DISTRIBUTOR: env.CONTRACT_REWARDS_DISTRIBUTOR || "",
  SUBSCRIPTION_BENEFITS: env.CONTRACT_SUBSCRIPTION_BENEFITS || "",
  SUBSCRIPTION_MANAGER: env.CONTRACT_SUBSCRIPTION_MANAGER || "",
  TARGETING_ENGINE: env.CONTRACT_TARGETING_ENGINE || "",
  TIMELOCK_EXECUTOR: env.CONTRACT_TIMELOCK_EXECUTOR || "",
  TOKEN_BRIDGE: env.CONTRACT_TOKEN_BRIDGE || "",
  TREASURY_MANAGER: env.CONTRACT_TREASURY_MANAGER || "",
  VESTING_SCHEDULE: env.CONTRACT_VESTING_SCHEDULE || "",
  WHITELIST_REGISTRY: env.CONTRACT_WHITELIST_REGISTRY || "",
  WRAPPED_TOKEN: env.CONTRACT_WRAPPED_TOKEN || "",
};

/**
 * Core contracts required for the platform to function.
 * All others are optional/feature-flagged.
 */
const REQUIRED_CONTRACT_ENV_VARS = [
  "CONTRACT_AD_REGISTRY",
  "CONTRACT_CAMPAIGN_ORCHESTRATOR",
  "CONTRACT_ESCROW_VAULT",
  "CONTRACT_FRAUD_PREVENTION",
  "CONTRACT_PAYMENT_PROCESSOR",
  "CONTRACT_AUCTION_ENGINE",
  "CONTRACT_REVENUE_SETTLEMENT",
  "CONTRACT_PUBLISHER_VERIFICATION",
  "CONTRACT_ANALYTICS_AGGREGATOR",
];

/**
 * Validates that all required contract IDs are present.
 * - In production: throws on any missing value, preventing startup.
 * - In development: logs a warning for each missing value so the server
 *   still starts (useful when only testing non-contract routes).
 * Pass SKIP_CONTRACT_VALIDATION=true to suppress even the warnings.
 */
export function validateContractIds(): void {
  if (process.env.SKIP_CONTRACT_VALIDATION === "true") {
    logger.warn(
      "[Config] Contract ID validation skipped (SKIP_CONTRACT_VALIDATION=true)",
    );
    return;
  }

  const missing = REQUIRED_CONTRACT_ENV_VARS.filter(
    (key) => !process.env[key] || process.env[key] === "PLACEHOLDER",
  );

  if (missing.length === 0) return;

  const message = `Missing or placeholder contract IDs:\n  ${missing.join("\n  ")}`;

  if (process.env.NODE_ENV === "production") {
    throw new Error(
      `[Config] ${message}\nSet these environment variables before starting the server.`,
    );
  }

  logger.warn(`[Config] ${message}`);
  logger.warn(
    "[Config] Contract calls will fail for the above contracts. Set SKIP_CONTRACT_VALIDATION=true to suppress this warning.",
  );
}
