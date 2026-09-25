// Network passphrases — copied from @stellar/stellar-sdk Networks to avoid
// importing the full Node.js SDK bundle in a shared config file.
const StellarNetworks = {
  PUBLIC: 'Public Global Stellar Network ; September 2015',
  TESTNET: 'Test SDF Network ; September 2015',
  FUTURENET: 'Test SDF Future Network ; October 2022',
} as const;

/**
 * Network Configuration for PulsarTrack on Stellar
 */

export const NETWORKS = {
  mainnet: {
    network: StellarNetworks.PUBLIC,
    horizonUrl: 'https://horizon.stellar.org',
    sorobanRpcUrl: 'https://mainnet.sorobanrpc.com',
    passphrase: StellarNetworks.PUBLIC,
  },
  testnet: {
    network: StellarNetworks.TESTNET,
    horizonUrl: 'https://horizon-testnet.stellar.org',
    sorobanRpcUrl: 'https://soroban-testnet.stellar.org',
    passphrase: StellarNetworks.TESTNET,
  },
  futurenet: {
    network: StellarNetworks.FUTURENET,
    horizonUrl: 'https://horizon-futurenet.stellar.org',
    sorobanRpcUrl: 'https://rpc-futurenet.stellar.org',
    passphrase: StellarNetworks.FUTURENET,
  },
} as const;

/**
 * Fallback source account used for read-only contract simulations when
 * NEXT_PUBLIC_SIMULATION_ACCOUNT is not set. Development only — production
 * callers must provide their own account via the environment variable.
 */
export const FALLBACK_SIMULATION_ACCOUNT =
  'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN';

export type NetworkType = keyof typeof NETWORKS;

export const CURRENT_NETWORK: NetworkType =
  (process.env.NEXT_PUBLIC_NETWORK as NetworkType) || 'testnet';

export const NETWORK_CONFIG = NETWORKS[CURRENT_NETWORK];

/**
 * Deployed PulsarTrack Soroban Contract IDs
 */
export const CONTRACT_IDS = {
  AD_REGISTRY: process.env.NEXT_PUBLIC_CONTRACT_AD_REGISTRY || '',
  CAMPAIGN_ORCHESTRATOR: process.env.NEXT_PUBLIC_CONTRACT_CAMPAIGN_ORCHESTRATOR || '',
  ESCROW_VAULT: process.env.NEXT_PUBLIC_CONTRACT_ESCROW_VAULT || '',
  FRAUD_PREVENTION: process.env.NEXT_PUBLIC_CONTRACT_FRAUD_PREVENTION || '',
  PAYMENT_PROCESSOR: process.env.NEXT_PUBLIC_CONTRACT_PAYMENT_PROCESSOR || '',
  GOVERNANCE_TOKEN: process.env.NEXT_PUBLIC_CONTRACT_GOVERNANCE_TOKEN || '',
  GOVERNANCE_DAO: process.env.NEXT_PUBLIC_CONTRACT_GOVERNANCE_DAO || '',
  PUBLISHER_VERIFICATION: process.env.NEXT_PUBLIC_CONTRACT_PUBLISHER_VERIFICATION || '',
  PUBLISHER_REPUTATION: process.env.NEXT_PUBLIC_CONTRACT_PUBLISHER_REPUTATION || '',
  ANALYTICS_AGGREGATOR: process.env.NEXT_PUBLIC_CONTRACT_ANALYTICS_AGGREGATOR || '',
  AUCTION_ENGINE: process.env.NEXT_PUBLIC_CONTRACT_AUCTION_ENGINE || '',
  SUBSCRIPTION_MANAGER: process.env.NEXT_PUBLIC_CONTRACT_SUBSCRIPTION_MANAGER || '',
  PRIVACY_LAYER: process.env.NEXT_PUBLIC_CONTRACT_PRIVACY_LAYER || '',
  TARGETING_ENGINE: process.env.NEXT_PUBLIC_CONTRACT_TARGETING_ENGINE || '',
  IDENTITY_REGISTRY: process.env.NEXT_PUBLIC_CONTRACT_IDENTITY_REGISTRY || '',
  DISPUTE_RESOLUTION: process.env.NEXT_PUBLIC_CONTRACT_DISPUTE_RESOLUTION || '',
  REVENUE_SETTLEMENT: process.env.NEXT_PUBLIC_CONTRACT_REVENUE_SETTLEMENT || '',
  REWARDS_DISTRIBUTOR: process.env.NEXT_PUBLIC_CONTRACT_REWARDS_DISTRIBUTOR || '',
} as const;

/**
 * Validates that all required contract IDs are present.
 * Throws in production, warns in development.
 */
function validateContractIds() {
  const missing = Object.entries(CONTRACT_IDS)
    .filter(([, id]) => !id)
    .map(([name]) => name);

  if (missing.length > 0) {
    const message = `Deployment Error: Missing contract IDs: ${missing.join(', ')}. Ensure they are set in .env.local`;

    if (process.env.NODE_ENV === 'production') {
      throw new Error(message);
    } else {
      console.warn(message);
    }
  }
}

// Run validation client-side only — contract calls never happen during SSG/SSR,
// and throwing at module evaluation time breaks `next build`.
if (typeof window !== 'undefined') {
  validateContractIds();
}

export type ContractName = keyof typeof CONTRACT_IDS;

/**
 * App details for wallet integration
 */
export const APP_DETAILS = {
  name: 'PulsarTrack',
  icon: typeof window !== 'undefined'
    ? `${window.location.origin}/logo.png`
    : '/logo.png',
};

/**
 * Stellar Lumens Conversion (1 XLM = 10,000,000 stroops)
 */
export const STROOPS_PER_XLM = 10_000_000;

export function stroopsToXlm(stroops: bigint | number): string {
  const stroopsNum = typeof stroops === 'bigint' ? stroops : BigInt(Math.floor(Number(stroops)));
  const xlmWhole = stroopsNum / BigInt(STROOPS_PER_XLM);
  const stroopsFraction = stroopsNum % BigInt(STROOPS_PER_XLM);

  if (stroopsFraction === 0n) {
    return xlmWhole.toString();
  }

  const fractionStr = stroopsFraction.toString().padStart(7, '0').replace(/0+$/, '');
  return `${xlmWhole}.${fractionStr}`;
}

export function xlmToStroops(xlm: number): bigint {
  if (!Number.isFinite(xlm)) {
    throw new Error(`Invalid XLM amount: ${xlm} is not a finite number`);
  }

  if (xlm < 0) {
    throw new Error(`Invalid XLM amount: ${xlm} cannot be negative`);
  }

  // Use toFixed(7) to get a fixed-point decimal string — toString() produces
  // exponent notation for values below 1e-7 (e.g. 0.0000001 → "1e-7"), which
  // breaks the split-on-dot logic below. toFixed(7) always produces a plain
  // decimal, e.g. "0.0000001". We round to 7 decimal places (the stroop
  // precision limit) so values like 0.12345678 become "0.1234568".
  const xlmStr = xlm.toFixed(7);
  const [whole, fraction] = xlmStr.split('.');

  if (fraction && fraction.length > 7) {
    throw new Error(`Invalid XLM amount: ${xlm} has more than 7 decimal places`);
  }

  const fractionPadded = (fraction || '').padEnd(7, '0');
  const stroopsStr = (whole || '0') + fractionPadded;

  return BigInt(stroopsStr);
}

/**
 * Ledger time constants (Stellar ~5s per ledger)
 */
export const LEDGER_TIME = {
  SECONDS_PER_LEDGER: 5,
  LEDGERS_PER_MINUTE: 12,
  LEDGERS_PER_HOUR: 720,
  LEDGERS_PER_DAY: 17280,
} as const;

export function isMainnet(): boolean {
  return CURRENT_NETWORK === 'mainnet';
}

export function getExplorerTxUrl(txHash: string): string {
  if (isMainnet()) {
    return `https://stellar.expert/explorer/public/tx/${txHash}`;
  }
  return `https://stellar.expert/explorer/testnet/tx/${txHash}`;
}

export function getExplorerAddressUrl(address: string): string {
  if (isMainnet()) {
    return `https://stellar.expert/explorer/public/account/${address}`;
  }
  return `https://stellar.expert/explorer/testnet/account/${address}`;
}

export function getExplorerContractUrl(contractId: string): string {
  if (isMainnet()) {
    return `https://stellar.expert/explorer/public/contract/${contractId}`;
  }
  return `https://stellar.expert/explorer/testnet/contract/${contractId}`;
}

export function getHorizonUrl(): string {
  return NETWORK_CONFIG.horizonUrl;
}

export function getSorobanRpcUrl(): string {
  return NETWORK_CONFIG.sorobanRpcUrl;
}

export function getNetworkPassphrase(): string {
  return NETWORK_CONFIG.passphrase;
}

// Required NEXT_PUBLIC_* env vars, validated at startup so a missing or
// malformed one fails loudly here instead of surfacing later inside
// whichever feature happens to need it.
export const REQUIRED_ENV_VARS = [
  'NEXT_PUBLIC_NETWORK',
  'NEXT_PUBLIC_WS_URL',
  'NEXT_PUBLIC_CONTRACT_AD_REGISTRY',
  'NEXT_PUBLIC_CONTRACT_ANALYTICS_AGGREGATOR',
  'NEXT_PUBLIC_CONTRACT_AUCTION_ENGINE',
  'NEXT_PUBLIC_CONTRACT_CAMPAIGN_ORCHESTRATOR',
  'NEXT_PUBLIC_CONTRACT_DISPUTE_RESOLUTION',
  'NEXT_PUBLIC_CONTRACT_ESCROW_VAULT',
  'NEXT_PUBLIC_CONTRACT_FRAUD_PREVENTION',
  'NEXT_PUBLIC_CONTRACT_GOVERNANCE_DAO',
  'NEXT_PUBLIC_CONTRACT_GOVERNANCE_TOKEN',
  'NEXT_PUBLIC_CONTRACT_IDENTITY_REGISTRY',
  'NEXT_PUBLIC_CONTRACT_PAYMENT_PROCESSOR',
  'NEXT_PUBLIC_CONTRACT_PRIVACY_LAYER',
  'NEXT_PUBLIC_CONTRACT_PUBLISHER_REPUTATION',
  'NEXT_PUBLIC_CONTRACT_PUBLISHER_VERIFICATION',
  'NEXT_PUBLIC_CONTRACT_REVENUE_SETTLEMENT',
  'NEXT_PUBLIC_CONTRACT_REWARDS_DISTRIBUTOR',
  'NEXT_PUBLIC_CONTRACT_SUBSCRIPTION_MANAGER',
  'NEXT_PUBLIC_CONTRACT_TARGETING_ENGINE',
] as const;

/**
 * Verifies every required NEXT_PUBLIC_* env var is present. Throws naming
 * the specific missing variable(s) instead of letting each feature fail
 * separately whenever it happens to be used.
 */
export function validateRequiredEnv(): void {
  const missing = REQUIRED_ENV_VARS.filter((key) => !process.env[key]);
  if (missing.length > 0) {
    throw new Error(
      `Missing required environment variable(s): ${missing.join(', ')}`
    );
  }
}

// validateRequiredEnv() is intentionally NOT called at module-load time.
// Calling it unconditionally here caused `next build` to fail during static
// pre-rendering of pages like /_not-found (which run with NODE_ENV=production
// but have no contract addresses available in CI or fresh contributor setups).
// Call validateRequiredEnv() explicitly from your server-startup path or from
// a server-side route handler that actually needs contract addresses, so the
// check only fires for real requests — not during static generation.
