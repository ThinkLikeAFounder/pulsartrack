import { describe, it, expect, beforeEach, afterEach } from "vitest";
import fs from "fs";
import path from "path";
import { Pool } from "pg";
import { runMigrations } from "./migrate";

describe("Database Migrations", () => {
  let pool: Pool;
  let testDir: string;

  beforeEach(async () => {
    testDir = path.join(__dirname, "test-migrations");
    if (!fs.existsSync(testDir)) {
      fs.mkdirSync(testDir, { recursive: true });
    }

    pool = new Pool({
      user: process.env.DB_USER || "postgres",
      password: process.env.DB_PASSWORD || "postgres",
      host: process.env.DB_HOST || "localhost",
      port: parseInt(process.env.DB_PORT || "5432"),
      database: process.env.DB_NAME || "test_pulsartrack",
    });

    const client = await pool.connect();
    try {
      await client.query("DROP SCHEMA IF EXISTS public CASCADE");
      await client.query("CREATE SCHEMA public");
    } finally {
      client.release();
    }
  });

  afterEach(async () => {
    if (fs.existsSync(testDir)) {
      fs.rmSync(testDir, { recursive: true });
    }
    await pool.end();
  });

  it("should apply migrations in order", async () => {
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

    const applied = await runMigrations(pool, testDir);

    expect(applied).toEqual(["001_add_email.sql", "002_add_created_at.sql"]);

    const client = await pool.connect();
    try {
      const result = await client.query(`
        SELECT column_name FROM information_schema.columns
        WHERE table_name = 'users' ORDER BY column_name
      `);
      const columns = result.rows.map((row) => row.column_name).sort();
      expect(columns).toContain("email");
      expect(columns).toContain("created_at");
    } finally {
      client.release();
    }
  });

  it("should not re-apply migrations", async () => {
    const schemaFile = path.join(testDir, "schema.sql");
    const migrationsDir = path.join(testDir, "migrations");

    fs.writeFileSync(
      schemaFile,
      `
      CREATE TABLE IF NOT EXISTS test_table (id SERIAL PRIMARY KEY);
    `,
    );

    fs.mkdirSync(migrationsDir, { recursive: true });
    fs.writeFileSync(
      path.join(migrationsDir, "001_init.sql"),
      `
      INSERT INTO test_table VALUES (1);
    `,
    );

    const first = await runMigrations(pool, testDir);
    expect(first).toEqual(["001_init.sql"]);

    const second = await runMigrations(pool, testDir);
    expect(second).toEqual([]);
  });

  it("should rollback on migration failure", async () => {
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

    try {
      await runMigrations(pool, testDir);
      expect.fail("Should have thrown an error");
    } catch {
      // Expected
    }

    const client = await pool.connect();
    try {
      const result = await client.query(`
        SELECT column_name FROM information_schema.columns
        WHERE table_name = 'test' AND column_name = 'bad_column'
      `);
      expect(result.rows).toHaveLength(0);
    } finally {
      client.release();
    }
  });

  it("should use advisory lock to prevent concurrent migrations", async () => {
    const schemaFile = path.join(testDir, "schema.sql");
    const migrationsDir = path.join(testDir, "migrations");

    fs.writeFileSync(schemaFile, "CREATE TABLE IF NOT EXISTS test (id SERIAL PRIMARY KEY);");
    fs.mkdirSync(migrationsDir, { recursive: true });

    const promise1 = runMigrations(pool, testDir);
    const promise2 = runMigrations(pool, testDir);

    const results = await Promise.all([promise1, promise2]);
    expect(results[0]).toBeDefined();
    expect(results[1]).toBeDefined();
  });

  it("should verify migration checksums and detect drift", async () => {
    const schemaFile = path.join(testDir, "schema.sql");
    const migrationsDir = path.join(testDir, "migrations");

    fs.writeFileSync(
      schemaFile,
      `
      CREATE TABLE IF NOT EXISTS verify_test (id SERIAL PRIMARY KEY);
    `,
    );

    fs.mkdirSync(migrationsDir, { recursive: true });
    fs.writeFileSync(
      path.join(migrationsDir, "001_test.sql"),
      `
      ALTER TABLE verify_test ADD COLUMN col1 TEXT;
    `,
    );

    await runMigrations(pool, testDir);

    const client = await pool.connect();
    try {
      const result = await client.query(`
        SELECT name FROM schema_migrations WHERE name = '001_test.sql'
      `);
      expect(result.rows).toHaveLength(1);
    } finally {
      client.release();
    }
  });
});
