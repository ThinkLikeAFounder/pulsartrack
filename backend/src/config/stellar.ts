import { Horizon, Networks } from "@stellar/stellar-sdk";
import { logger } from "../lib/logger";

const NETWORK = process.env.STELLAR_NETWORK || "testnet";
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

export const STELLAR_REQUEST_TIMEOUT_MS = Number.parseInt(
  process.env.STELLAR_REQUEST_TIMEOUT_MS || "15000",
  10,
);

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
  ACCESS_CONTROL: process.env.CONTRACT_ACCESS_CONTROL || "",
  AD_REGISTRY: process.env.CONTRACT_AD_REGISTRY || "",
  ANALYTICS_AGGREGATOR: process.env.CONTRACT_ANALYTICS_AGGREGATOR || "",
  ANOMALY_DETECTOR: process.env.CONTRACT_ANOMALY_DETECTOR || "",
  AUCTION_ENGINE: process.env.CONTRACT_AUCTION_ENGINE || "",
  AUDIENCE_SEGMENTS: process.env.CONTRACT_AUDIENCE_SEGMENTS || "",
  BUDGET_OPTIMIZER: process.env.CONTRACT_BUDGET_OPTIMIZER || "",
  CAMPAIGN_ANALYTICS: process.env.CONTRACT_CAMPAIGN_ANALYTICS || "",
  CAMPAIGN_LIFECYCLE: process.env.CONTRACT_CAMPAIGN_LIFECYCLE || "",
  CAMPAIGN_ORCHESTRATOR: process.env.CONTRACT_CAMPAIGN_ORCHESTRATOR || "",
  CREATIVE_MARKETPLACE: process.env.CONTRACT_CREATIVE_MARKETPLACE || "",
  DISPUTE_RESOLUTION: process.env.CONTRACT_DISPUTE_RESOLUTION || "",
  ESCROW_VAULT: process.env.CONTRACT_ESCROW_VAULT || "",
  FRAUD_PREVENTION: process.env.CONTRACT_FRAUD_PREVENTION || "",
  GOVERNANCE_CORE: process.env.CONTRACT_GOVERNANCE_CORE || "",
  GOVERNANCE_DAO: process.env.CONTRACT_GOVERNANCE_DAO || "",
  GOVERNANCE_TOKEN: process.env.CONTRACT_GOVERNANCE_TOKEN || "",
  IDENTITY_REGISTRY: process.env.CONTRACT_IDENTITY_REGISTRY || "",
  KYC_REGISTRY: process.env.CONTRACT_KYC_REGISTRY || "",
  LIQUIDITY_POOL: process.env.CONTRACT_LIQUIDITY_POOL || "",
  MILESTONE_TRACKER: process.env.CONTRACT_MILESTONE_TRACKER || "",
  MULTISIG_TREASURY: process.env.CONTRACT_MULTISIG_TREASURY || "",
  ORACLE_INTEGRATION: process.env.CONTRACT_ORACLE_INTEGRATION || "",
  PAYMENT_PROCESSOR: process.env.CONTRACT_PAYMENT_PROCESSOR || "",
  PAYOUT_AUTOMATION: process.env.CONTRACT_PAYOUT_AUTOMATION || "",
  PERFORMANCE_ORACLE: process.env.CONTRACT_PERFORMANCE_ORACLE || "",
  PRIVACY_LAYER: process.env.CONTRACT_PRIVACY_LAYER || "",
  PUBLISHER_NETWORK: process.env.CONTRACT_PUBLISHER_NETWORK || "",
  PUBLISHER_REPUTATION: process.env.CONTRACT_PUBLISHER_REPUTATION || "",
  PUBLISHER_VERIFICATION: process.env.CONTRACT_PUBLISHER_VERIFICATION || "",
  RECURRING_PAYMENT: process.env.CONTRACT_RECURRING_PAYMENT || "",
  REFUND_PROCESSOR: process.env.CONTRACT_REFUND_PROCESSOR || "",
  REVENUE_SETTLEMENT: process.env.CONTRACT_REVENUE_SETTLEMENT || "",
  REWARDS_DISTRIBUTOR: process.env.CONTRACT_REWARDS_DISTRIBUTOR || "",
  SUBSCRIPTION_BENEFITS: process.env.CONTRACT_SUBSCRIPTION_BENEFITS || "",
  SUBSCRIPTION_MANAGER: process.env.CONTRACT_SUBSCRIPTION_MANAGER || "",
  TARGETING_ENGINE: process.env.CONTRACT_TARGETING_ENGINE || "",
  TIMELOCK_EXECUTOR: process.env.CONTRACT_TIMELOCK_EXECUTOR || "",
  TOKEN_BRIDGE: process.env.CONTRACT_TOKEN_BRIDGE || "",
  TREASURY_MANAGER: process.env.CONTRACT_TREASURY_MANAGER || "",
  VESTING_SCHEDULE: process.env.CONTRACT_VESTING_SCHEDULE || "",
  WHITELIST_REGISTRY: process.env.CONTRACT_WHITELIST_REGISTRY || "",
  WRAPPED_TOKEN: process.env.CONTRACT_WRAPPED_TOKEN || "",
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
