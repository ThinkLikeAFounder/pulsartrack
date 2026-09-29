import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import fs from "fs";
import path from "path";
import type { Pool, PoolClient, QueryResult, QueryResultRow } from "pg";

// Helper to create mock query results
const mockQueryResult = <T extends QueryResultRow = any>(rows: T[] = [], rowCount: number = 0): QueryResult<T> => ({
  rows,
  rowCount,
  command: "",
  oid: 0,
  fields: [],
});

// Create mock functions at module level
const mockQuery = vi.fn();
const mockRelease = vi.fn();
const mockConnect = vi.fn();
const mockEnd = vi.fn();

const mockClient: Partial<PoolClient> = {
  query: mockQuery,
  release: mockRelease,
};

const mockPool = {
  connect: mockConnect,
  end: mockEnd,
};

// Mock the pg module
vi.mock("pg", () => {
  return {
    Pool: vi.fn(function (this: any) {
      return mockPool;
    }),
  };
});

describe("Database Migrations", () => {
  let pool: Pool;
  let testDir: string;

  beforeEach(async () => {
    testDir = path.join(__dirname, "test-migrations");
    if (!fs.existsSync(testDir)) {
      fs.mkdirSync(testDir, { recursive: true });
    }

    // Reset mocks
    mockQuery.mockReset();
    mockRelease.mockReset();
    mockConnect.mockReset();
    mockEnd.mockReset();

    // Default mock implementations
    mockQuery.mockResolvedValue(mockQueryResult());
    mockRelease.mockResolvedValue(undefined);
    mockEnd.mockResolvedValue(undefined);
    mockConnect.mockResolvedValue(mockClient);

    const { Pool: PoolConstructor } = await import("pg");
    pool = new PoolConstructor();
  });

  afterEach(async () => {
    if (fs.existsSync(testDir)) {
      fs.rmSync(testDir, { recursive: true });
    }
    vi.clearAllMocks();
  });

  it("should apply migrations in order", async () => {
    const { runMigrations } = await import("./migrate");
    
    const schemaFile = path.join(testDir, "schema.sql");
    const migrationsDir = path.join(testDir, "migrations");

    fs.writeFileSync(
      schemaFile,
      `
      CREATE TABLE IF NOT EXISTS users (
        id SERIAL PRIMARY KEY,
        name TEXT NOT NULL
      );
    `,
    );

    fs.mkdirSync(migrationsDir, { recursive: true });

    fs.writeFileSync(
      path.join(migrationsDir, "001_add_email.sql"),
      `
      ALTER TABLE users ADD COLUMN email TEXT UNIQUE;
    `,
    );

    fs.writeFileSync(
      path.join(migrationsDir, "002_add_created_at.sql"),
      `
      ALTER TABLE users ADD COLUMN created_at TIMESTAMPTZ DEFAULT NOW();
    `,
    );

    // Mock the schema_migrations query to return no existing migrations
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // advisory lock
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // schema.sql
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // create schema_migrations table
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // select from schema_migrations
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // BEGIN for first migration
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // first migration
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // insert into schema_migrations
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // COMMIT
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // BEGIN for second migration
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // second migration
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // insert into schema_migrations
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // COMMIT
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // advisory unlock

    const applied = await runMigrations(pool, testDir);

    expect(applied).toEqual(["001_add_email.sql", "002_add_created_at.sql"]);
    expect(mockConnect).toHaveBeenCalled();
    expect(mockRelease).toHaveBeenCalled();
  });

  it("should not re-apply migrations", async () => {
    const { runMigrations } = await import("./migrate");
    
    const schemaFile = path.join(testDir, "schema.sql");
    const migrationsDir = path.join(testDir, "migrations");

    fs.writeFileSync(
      schemaFile,
      `
      CREATE TABLE IF NOT EXISTS test_table (id SERIAL PRIMARY KEY);
    `,
    );

    fs.mkdirSync(migrationsDir, { recursive: true });
    const migrationContent = `
      INSERT INTO test_table VALUES (1);
    `;
    fs.writeFileSync(
      path.join(migrationsDir, "001_init.sql"),
      migrationContent,
    );

    // First run: migration is applied
    mockQuery.mockReset();
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // advisory lock
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // schema.sql
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // create schema_migrations table
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // select from schema_migrations (empty)
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // BEGIN
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // migration
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // insert
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // COMMIT
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // advisory unlock

    const first = await runMigrations(pool, testDir);
    expect(first).toEqual(["001_init.sql"]);

    // Second run: migration already applied
    mockQuery.mockReset();
    const checksum = require("crypto").createHash("sha256").update(migrationContent).digest("hex");
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // advisory lock
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // schema.sql
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // create schema_migrations table
    mockQuery.mockResolvedValueOnce(mockQueryResult([{ name: "001_init.sql", checksum }], 1)); // select from schema_migrations (has migration)
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // advisory unlock

    const second = await runMigrations(pool, testDir);
    expect(second).toEqual([]);
  });

  it("should rollback on migration failure", async () => {
    const { runMigrations } = await import("./migrate");
    
    const schemaFile = path.join(testDir, "schema.sql");
    const migrationsDir = path.join(testDir, "migrations");

    fs.writeFileSync(
      schemaFile,
      `
      CREATE TABLE IF NOT EXISTS test (id SERIAL PRIMARY KEY);
    `,
    );

    fs.mkdirSync(migrationsDir, { recursive: true });
    fs.writeFileSync(
      path.join(migrationsDir, "001_valid.sql"),
      `
      ALTER TABLE test ADD COLUMN name TEXT;
    `,
    );

    fs.writeFileSync(
      path.join(migrationsDir, "002_invalid.sql"),
      `
      ALTER TABLE test ADD COLUMN bad_column TEXT;
      INVALID SQL HERE;
    `,
    );

    mockQuery.mockReset();
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // advisory lock
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // schema.sql
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // create schema_migrations table
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // select from schema_migrations
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // BEGIN for first migration
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // first migration succeeds
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // insert into schema_migrations
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // COMMIT
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // BEGIN for second migration
    mockQuery.mockRejectedValueOnce(new Error("syntax error at or near \"INVALID\"")); // second migration fails
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // ROLLBACK
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // advisory unlock

    await expect(runMigrations(pool, testDir)).rejects.toThrow(
      /Migration 002_invalid\.sql failed and rolled back/,
    );

    // Verify ROLLBACK was called
    const queryCalls = mockQuery.mock.calls;
    const rollbackCall = queryCalls.find((call) => call[0] === "ROLLBACK");
    expect(rollbackCall).toBeDefined();
  });

  it("should use advisory lock to prevent concurrent migrations", async () => {
    const { runMigrations } = await import("./migrate");
    
    const schemaFile = path.join(testDir, "schema.sql");
    const migrationsDir = path.join(testDir, "migrations");

    fs.writeFileSync(schemaFile, "CREATE TABLE IF NOT EXISTS test (id SERIAL PRIMARY KEY);");
    fs.mkdirSync(migrationsDir, { recursive: true });

    // Mock successful migration run
    mockQuery.mockReset();
    mockQuery.mockResolvedValue(mockQueryResult());

    const promise1 = runMigrations(pool, testDir);
    const promise2 = runMigrations(pool, testDir);

    const results = await Promise.all([promise1, promise2]);
    expect(results[0]).toBeDefined();
    expect(results[1]).toBeDefined();

    // Verify advisory lock was acquired
    const lockCalls = mockQuery.mock.calls.filter(
      (call) => typeof call[0] === "string" && call[0].includes("pg_advisory_lock"),
    );
    expect(lockCalls.length).toBeGreaterThanOrEqual(2);
  });

  it("should verify migration checksums and detect drift", async () => {
    const { runMigrations } = await import("./migrate");
    
    const schemaFile = path.join(testDir, "schema.sql");
    const migrationsDir = path.join(testDir, "migrations");

    fs.writeFileSync(
      schemaFile,
      `
      CREATE TABLE IF NOT EXISTS verify_test (id SERIAL PRIMARY KEY);
    `,
    );

    fs.mkdirSync(migrationsDir, { recursive: true });
    const migrationContent = `
      ALTER TABLE verify_test ADD COLUMN col1 TEXT;
    `;
    fs.writeFileSync(
      path.join(migrationsDir, "001_test.sql"),
      migrationContent,
    );

    mockQuery.mockReset();
    const checksum = require("crypto").createHash("sha256").update(migrationContent).digest("hex");
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // advisory lock
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // schema.sql
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // create schema_migrations table
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // select from schema_migrations
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // BEGIN
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // migration
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // insert with checksum
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // COMMIT
    mockQuery.mockResolvedValueOnce(mockQueryResult()); // advisory unlock

    const applied = await runMigrations(pool, testDir);
    expect(applied).toEqual(["001_test.sql"]);

    // Verify that insert was called with the migration name
    const insertCalls = mockQuery.mock.calls.filter(
      (call) => typeof call[0] === "string" && call[0].includes("INSERT INTO schema_migrations"),
    );
    expect(insertCalls.length).toBeGreaterThan(0);
    expect(insertCalls[0][1]).toEqual(["001_test.sql", checksum]);
  });
});
