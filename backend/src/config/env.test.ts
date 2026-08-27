import { describe, it, expect } from "vitest";
import {
  EnvValidationError,
  formatIssues,
  loadEnv,
  validateEnv,
} from "./env";

const VALID_ACCOUNT = `G${"A".repeat(55)}`;
const VALID_CONTRACT = `C${"A".repeat(55)}`;

const CONTRACT_VARS = [
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
];

function productionEnv(overrides: Record<string, string> = {}) {
  const base: Record<string, string> = {
    NODE_ENV: "production",
    PORT: "4000",
    JWT_SECRET: "a".repeat(48),
    DATABASE_URL: "postgresql://user:pass@db:5432/pulsartrack",
    DB_HOST: "db",
    DB_PORT: "5432",
    DB_NAME: "pulsartrack",
    DB_USER: "pulsartrack",
    DB_PASSWORD: "s3cret",
    REDIS_URL: "redis://cache:6379",
    STELLAR_NETWORK: "mainnet",
    SIMULATION_ACCOUNT: VALID_ACCOUNT,
    CORS_ORIGIN: "https://app.pulsartrack.io",
  };
  for (const name of CONTRACT_VARS) base[name] = VALID_CONTRACT;
  return { ...base, ...overrides } as NodeJS.ProcessEnv;
}

function issueFor(env: NodeJS.ProcessEnv, variable: string) {
  return validateEnv(env).issues.find((i) => i.variable === variable);
}

describe("validateEnv", () => {
  it("accepts a fully configured production environment", () => {
    const { issues, values } = validateEnv(productionEnv());

    expect(issues).toEqual([]);
    expect(values.NODE_ENV).toBe("production");
    expect(values.PORT).toBe(4000);
    expect(values.DB_PORT).toBe(5432);
  });

  it("applies defaults for optional variables", () => {
    const { values } = validateEnv(
      productionEnv({ PORT: "", LOG_LEVEL: "", STELLAR_REQUEST_TIMEOUT_MS: "" }),
    );

    expect(values.PORT).toBe(4000);
    expect(values.LOG_LEVEL).toBe("info");
    expect(values.STELLAR_REQUEST_TIMEOUT_MS).toBe(15000);
  });

  it("reports every missing production variable at once, naming each one", () => {
    const env = productionEnv({
      JWT_SECRET: "",
      DB_PASSWORD: "",
      REDIS_URL: "",
    });

    const names = validateEnv(env).issues.map((i) => i.variable);

    expect(names).toEqual(
      expect.arrayContaining(["JWT_SECRET", "DB_PASSWORD", "REDIS_URL"]),
    );
    expect(names).toHaveLength(3);
  });

  it("rejects a JWT_SECRET that is too short", () => {
    const issue = issueFor(productionEnv({ JWT_SECRET: "short" }), "JWT_SECRET");
    expect(issue?.message).toContain("at least 32 characters");
  });

  it("rejects a non-numeric or out-of-range PORT", () => {
    expect(issueFor(productionEnv({ PORT: "not-a-port" }), "PORT")).toBeDefined();
    expect(issueFor(productionEnv({ PORT: "70000" }), "PORT")).toBeDefined();
  });

  it("rejects a DATABASE_URL with the wrong protocol", () => {
    const issue = issueFor(
      productionEnv({ DATABASE_URL: "mysql://user:pass@db:3306/pulsartrack" }),
      "DATABASE_URL",
    );
    expect(issue?.message).toContain("postgresql:");
  });

  it("rejects an unknown STELLAR_NETWORK", () => {
    const issue = issueFor(
      productionEnv({ STELLAR_NETWORK: "futurenet" }),
      "STELLAR_NETWORK",
    );
    expect(issue?.message).toContain("testnet");
  });

  it("rejects a malformed SIMULATION_ACCOUNT", () => {
    const issue = issueFor(
      productionEnv({ SIMULATION_ACCOUNT: "GNOTAREALACCOUNT" }),
      "SIMULATION_ACCOUNT",
    );
    expect(issue?.message).toContain("Stellar account ID");
  });

  it("rejects a malformed contract ID", () => {
    const issue = issueFor(
      productionEnv({ CONTRACT_ESCROW_VAULT: "not-a-contract" }),
      "CONTRACT_ESCROW_VAULT",
    );
    expect(issue?.message).toContain("contract ID");
  });

  it("requires contract IDs in production", () => {
    const issue = issueFor(
      productionEnv({ CONTRACT_ESCROW_VAULT: "" }),
      "CONTRACT_ESCROW_VAULT",
    );
    expect(issue?.message).toContain("required in production");
  });

  it("honours SKIP_CONTRACT_VALIDATION for unset contract IDs", () => {
    const env = productionEnv({
      CONTRACT_ESCROW_VAULT: "",
      SKIP_CONTRACT_VALIDATION: "true",
    });
    expect(validateEnv(env).issues).toEqual([]);
  });

  it("does not require deploy-time secrets in development", () => {
    const { issues } = validateEnv({
      NODE_ENV: "development",
    } as NodeJS.ProcessEnv);

    expect(issues).toEqual([]);
  });

  it("still rejects malformed values in development", () => {
    const issue = issueFor(
      { NODE_ENV: "development", REDIS_URL: "http://cache:6379" } as NodeJS.ProcessEnv,
      "REDIS_URL",
    );
    expect(issue).toBeDefined();
  });
});

describe("loadEnv", () => {
  it("returns validated values when the environment is valid", () => {
    expect(loadEnv(productionEnv()).PORT).toBe(4000);
  });

  it("throws EnvValidationError naming the offending variables", () => {
    const env = productionEnv({ JWT_SECRET: "", DB_HOST: "" });

    try {
      loadEnv(env);
      expect.unreachable("loadEnv should have thrown");
    } catch (err) {
      expect(err).toBeInstanceOf(EnvValidationError);
      const typed = err as EnvValidationError;
      expect(typed.issues.map((i) => i.variable)).toEqual([
        "JWT_SECRET",
        "DB_HOST",
      ]);
      expect(typed.message).toContain("JWT_SECRET");
      expect(typed.message).toContain("DB_HOST");
    }
  });
});

describe("formatIssues", () => {
  it("lists one line per problem and points at .env.example", () => {
    const message = formatIssues([
      { variable: "JWT_SECRET", message: "is required" },
    ]);

    expect(message).toContain("1 problem found");
    expect(message).toContain("- JWT_SECRET is required");
    expect(message).toContain(".env.example");
  });
});
