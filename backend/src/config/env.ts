/**
 * Startup environment-variable validation.
 *
 * Every required variable is checked once, before the HTTP server starts
 * listening, so a misconfigured deploy fails immediately with a message
 * naming exactly which variables are wrong — instead of failing later, at
 * request time, in whichever route first touches the bad config.
 */

export type EnvMode = "development" | "test" | "production";

interface RuleBase {
  /** Variables that are required only in some modes. */
  requiredIn?: EnvMode[];
  /** Used when the variable is absent and not required. */
  default?: string;
  description?: string;
}

interface StringRule extends RuleBase {
  type: "string";
  minLength?: number;
  pattern?: RegExp;
  patternHint?: string;
  oneOf?: readonly string[];
}

interface IntRule extends RuleBase {
  type: "int";
  min?: number;
  max?: number;
}

interface UrlRule extends RuleBase {
  type: "url";
  protocols?: readonly string[];
}

type Rule = StringRule | IntRule | UrlRule;

const ALL_MODES: EnvMode[] = ["development", "test", "production"];

/** Stellar public keys / contract IDs are 56-char base32 strings. */
const STELLAR_ACCOUNT = /^G[A-Z2-7]{55}$/;
const CONTRACT_ID = /^C[A-Z2-7]{55}$/;

const SCHEMA: Record<string, Rule> = {
  NODE_ENV: {
    type: "string",
    oneOf: ALL_MODES,
    default: "development",
    description: "Runtime mode",
  },
  PORT: {
    type: "int",
    min: 1,
    max: 65535,
    default: "4000",
    description: "HTTP port the API listens on",
  },
  JWT_SECRET: {
    type: "string",
    minLength: 32,
    requiredIn: ["production"],
    description: "HMAC secret for API tokens (>= 32 chars)",
  },
  DATABASE_URL: {
    type: "url",
    protocols: ["postgresql:", "postgres:"],
    requiredIn: ["production"],
    description: "Prisma PostgreSQL connection string",
  },
  DB_HOST: {
    type: "string",
    minLength: 1,
    requiredIn: ["production"],
    description: "PostgreSQL host for the pg pool",
  },
  DB_PORT: {
    type: "int",
    min: 1,
    max: 65535,
    default: "5432",
    description: "PostgreSQL port",
  },
  DB_NAME: {
    type: "string",
    minLength: 1,
    requiredIn: ["production"],
    description: "PostgreSQL database name",
  },
  DB_USER: {
    type: "string",
    minLength: 1,
    requiredIn: ["production"],
    description: "PostgreSQL user",
  },
  DB_PASSWORD: {
    type: "string",
    minLength: 1,
    requiredIn: ["production"],
    description: "PostgreSQL password",
  },
  REDIS_URL: {
    type: "url",
    protocols: ["redis:", "rediss:"],
    requiredIn: ["production"],
    description: "Redis connection string for rate limiting and WS sessions",
  },
  STELLAR_NETWORK: {
    type: "string",
    oneOf: ["testnet", "mainnet"],
    default: "testnet",
    description: "Stellar network to target",
  },
  SIMULATION_ACCOUNT: {
    type: "string",
    pattern: STELLAR_ACCOUNT,
    patternHint: "a 56-character Stellar account ID starting with G",
    requiredIn: ["production"],
    description: "Account used for read-only contract simulations",
  },
  STELLAR_REQUEST_TIMEOUT_MS: {
    type: "int",
    min: 1,
    default: "15000",
    description: "Horizon/Soroban RPC request timeout",
  },
  EXPRESS_RESPONSE_TIMEOUT_MS: {
    type: "int",
    min: 1,
    default: "30000",
    description: "Express response timeout",
  },
  CORS_ORIGIN: {
    type: "string",
    minLength: 1,
    requiredIn: ["production"],
    description: "Allowed browser origin(s)",
  },
  LOG_LEVEL: {
    type: "string",
    oneOf: ["fatal", "error", "warn", "info", "debug", "trace", "silent"],
    default: "info",
    description: "Pino log level",
  },
};

/**
 * Contract IDs the backend reads. In production every one of these must be a
 * real deployed contract address; elsewhere an unset value is tolerated so the
 * API can boot against a partial local deployment.
 */
const CONTRACT_ENV_VARS = [
  "CONTRACT_ACCESS_CONTROL",
  "CONTRACT_AD_REGISTRY",
  "CONTRACT_ANALYTICS_AGGREGATOR",
  "CONTRACT_ANOMALY_DETECTOR",
  "CONTRACT_AUCTION_ENGINE",
  "CONTRACT_AUDIENCE_SEGMENTS",
  "CONTRACT_BUDGET_OPTIMIZER",
  "CONTRACT_CAMPAIGN_ANALYTICS",
  "CONTRACT_CAMPAIGN_LIFECYCLE",
  "CONTRACT_CAMPAIGN_ORCHESTRATOR",
  "CONTRACT_CREATIVE_MARKETPLACE",
  "CONTRACT_DISPUTE_RESOLUTION",
  "CONTRACT_ESCROW_VAULT",
  "CONTRACT_FRAUD_PREVENTION",
  "CONTRACT_GOVERNANCE_CORE",
  "CONTRACT_GOVERNANCE_DAO",
  "CONTRACT_GOVERNANCE_TOKEN",
  "CONTRACT_IDENTITY_REGISTRY",
  "CONTRACT_KYC_REGISTRY",
  "CONTRACT_LIQUIDITY_POOL",
  "CONTRACT_MILESTONE_TRACKER",
  "CONTRACT_MULTISIG_TREASURY",
  "CONTRACT_ORACLE_INTEGRATION",
  "CONTRACT_PAYMENT_PROCESSOR",
  "CONTRACT_PAYOUT_AUTOMATION",
  "CONTRACT_PERFORMANCE_ORACLE",
  "CONTRACT_PRIVACY_LAYER",
  "CONTRACT_PUBLISHER_NETWORK",
  "CONTRACT_PUBLISHER_REPUTATION",
  "CONTRACT_PUBLISHER_VERIFICATION",
  "CONTRACT_RECURRING_PAYMENT",
  "CONTRACT_REFUND_PROCESSOR",
  "CONTRACT_REVENUE_SETTLEMENT",
  "CONTRACT_REWARDS_DISTRIBUTOR",
  "CONTRACT_SUBSCRIPTION_BENEFITS",
  "CONTRACT_SUBSCRIPTION_MANAGER",
  "CONTRACT_TARGETING_ENGINE",
  "CONTRACT_TIMELOCK_EXECUTOR",
  "CONTRACT_TOKEN_BRIDGE",
  "CONTRACT_TREASURY_MANAGER",
  "CONTRACT_VESTING_SCHEDULE",
  "CONTRACT_WHITELIST_REGISTRY",
  "CONTRACT_WRAPPED_TOKEN",
] as const;

export interface EnvIssue {
  variable: string;
  message: string;
}

export interface ValidatedEnv {
  NODE_ENV: EnvMode;
  PORT: number;
  JWT_SECRET?: string;
  DATABASE_URL?: string;
  DB_HOST?: string;
  DB_PORT: number;
  DB_NAME?: string;
  DB_USER?: string;
  DB_PASSWORD?: string;
  REDIS_URL?: string;
  STELLAR_NETWORK: string;
  SIMULATION_ACCOUNT?: string;
  STELLAR_REQUEST_TIMEOUT_MS: number;
  EXPRESS_RESPONSE_TIMEOUT_MS: number;
  CORS_ORIGIN?: string;
  LOG_LEVEL: string;
  [key: string]: string | number | undefined;
}

export interface EnvValidationResult {
  values: ValidatedEnv;
  issues: EnvIssue[];
}

function describeRule(rule: Rule): string {
  switch (rule.type) {
    case "int": {
      const bounds: string[] = [];
      if (rule.min !== undefined) bounds.push(`>= ${rule.min}`);
      if (rule.max !== undefined) bounds.push(`<= ${rule.max}`);
      return bounds.length ? `an integer ${bounds.join(" and ")}` : "an integer";
    }
    case "url":
      return rule.protocols
        ? `a URL using one of: ${rule.protocols.join(", ")}`
        : "a URL";
    default:
      if (rule.oneOf) return `one of: ${rule.oneOf.join(", ")}`;
      if (rule.patternHint) return rule.patternHint;
      if (rule.minLength) return `at least ${rule.minLength} characters`;
      return "a non-empty string";
  }
}

function checkValue(rule: Rule, raw: string): string | null {
  switch (rule.type) {
    case "int": {
      if (!/^-?\d+$/.test(raw)) return `must be ${describeRule(rule)}`;
      const parsed = Number.parseInt(raw, 10);
      if (rule.min !== undefined && parsed < rule.min)
        return `must be ${describeRule(rule)}`;
      if (rule.max !== undefined && parsed > rule.max)
        return `must be ${describeRule(rule)}`;
      return null;
    }
    case "url": {
      let parsed: URL;
      try {
        parsed = new URL(raw);
      } catch {
        return `must be ${describeRule(rule)}`;
      }
      if (rule.protocols && !rule.protocols.includes(parsed.protocol))
        return `must be ${describeRule(rule)}`;
      return null;
    }
    default: {
      if (rule.oneOf && !rule.oneOf.includes(raw))
        return `must be ${describeRule(rule)} (got "${raw}")`;
      if (rule.minLength !== undefined && raw.length < rule.minLength)
        return `must be ${describeRule(rule)}`;
      if (rule.pattern && !rule.pattern.test(raw))
        return `must be ${describeRule(rule)}`;
      return null;
    }
  }
}

function resolveMode(source: NodeJS.ProcessEnv): EnvMode {
  const raw = source.NODE_ENV?.trim();
  return raw === "production" || raw === "test" ? raw : "development";
}

/**
 * Validates the environment without touching process state. Returns the
 * coerced values plus every issue found, so a caller can report all
 * misconfigured variables at once rather than one per restart.
 */
export function validateEnv(
  source: NodeJS.ProcessEnv = process.env,
): EnvValidationResult {
  const mode = resolveMode(source);
  const issues: EnvIssue[] = [];
  const values: Record<string, string | number | undefined> = {};

  for (const [name, rule] of Object.entries(SCHEMA)) {
    const raw = source[name]?.trim();
    // A variable with a default is never "missing" — the default stands in.
    const required =
      rule.default === undefined &&
      (rule.requiredIn ?? ALL_MODES).includes(mode);

    if (!raw) {
      if (required) {
        issues.push({
          variable: name,
          message: `is required in ${mode} but is missing or empty — expected ${describeRule(rule)}${
            rule.description ? ` (${rule.description})` : ""
          }`,
        });
      }
      if (rule.default !== undefined) {
        values[name] =
          rule.type === "int"
            ? Number.parseInt(rule.default, 10)
            : rule.default;
      }
      continue;
    }

    const problem = checkValue(rule, raw);
    if (problem) {
      issues.push({
        variable: name,
        message: `${problem}${rule.description ? ` (${rule.description})` : ""}`,
      });
      continue;
    }

    values[name] = rule.type === "int" ? Number.parseInt(raw, 10) : raw;
  }

  // Contract IDs: format-checked whenever set, and required in production.
  const skipContracts = source.SKIP_CONTRACT_VALIDATION === "true";
  for (const name of CONTRACT_ENV_VARS) {
    const raw = source[name]?.trim();
    if (!raw) {
      if (mode === "production" && !skipContracts) {
        issues.push({
          variable: name,
          message:
            "is required in production but is missing or empty — expected a deployed contract ID starting with C",
        });
      }
      continue;
    }
    if (!CONTRACT_ID.test(raw)) {
      issues.push({
        variable: name,
        message:
          "must be a 56-character Stellar contract ID starting with C (got a malformed value)",
      });
      continue;
    }
    values[name] = raw;
  }

  return { values: values as ValidatedEnv, issues };
}

export function formatIssues(issues: EnvIssue[]): string {
  const lines = issues.map((i) => `  - ${i.variable} ${i.message}`);
  return [
    `Invalid environment configuration — ${issues.length} problem${
      issues.length === 1 ? "" : "s"
    } found:`,
    ...lines,
    "",
    "See backend/.env.example for the expected values.",
  ].join("\n");
}

export class EnvValidationError extends Error {
  readonly issues: EnvIssue[];

  constructor(issues: EnvIssue[]) {
    super(formatIssues(issues));
    this.name = "EnvValidationError";
    this.issues = issues;
  }
}

/**
 * Validates the environment and throws {@link EnvValidationError} if anything
 * is missing or malformed. Call this once at startup, before the server
 * starts listening.
 */
export function loadEnv(source: NodeJS.ProcessEnv = process.env): ValidatedEnv {
  const { values, issues } = validateEnv(source);
  if (issues.length > 0) throw new EnvValidationError(issues);
  return values;
}
