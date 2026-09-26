import crypto from "crypto";
import fs from "fs";
import path from "path";
import type { Pool } from "pg";
import { logger } from "../lib/logger";

const MIGRATION_LOCK_ID = 872391;

function computeChecksum(content: string): string {
  return crypto.createHash("sha256").update(content).digest("hex");
}

/**
 * Brings the database up to date and returns the migrations it applied.
 *
 * `schema.sql` is the single source of truth for a fresh database; it is
 * written with IF NOT EXISTS so running it on an existing database changes
 * nothing. Changes to an existing database go in `migrations/*.sql`, applied
 * once each in filename order and recorded in `schema_migrations`.
 *
 * Verifies checksums to detect drift and uses transactional safety to rollback
 * failed migrations automatically.
 */
export async function runMigrations(pool: Pool, dir: string = __dirname): Promise<string[]> {
  const client = await pool.connect();
  const applied: string[] = [];
  try {
    await client.query("SELECT pg_advisory_lock($1)", [MIGRATION_LOCK_ID]);

    await client.query(fs.readFileSync(path.join(dir, "schema.sql"), "utf8"));
    await client.query(
      `CREATE TABLE IF NOT EXISTS schema_migrations (
         id SERIAL PRIMARY KEY,
         name TEXT UNIQUE NOT NULL,
         checksum TEXT NOT NULL,
         applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
       )`,
    );

    const migrationsDir = path.join(dir, "migrations");
    const files = fs.existsSync(migrationsDir)
      ? fs.readdirSync(migrationsDir).filter((file) => file.endsWith(".sql")).sort()
      : [];

    const applied_migrations = await client.query<{ name: string; checksum: string }>(
      "SELECT name, checksum FROM schema_migrations ORDER BY id",
    );
    const doneMap = new Map(applied_migrations.rows.map((row) => [row.name, row.checksum]));

    for (const file of files) {
      const filePath = path.join(migrationsDir, file);
      const content = fs.readFileSync(filePath, "utf8");
      const checksum = computeChecksum(content);

      if (doneMap.has(file)) {
        const storedChecksum = doneMap.get(file)!;
        if (storedChecksum !== checksum) {
          throw new Error(
            `Migration drift detected in ${file}: stored checksum differs. ` +
              `This indicates the file has been modified after application. ` +
              `Do not modify applied migrations.`,
          );
        }
        continue;
      }

      await client.query("BEGIN");
      try {
        await client.query(content);
        await client.query(
          "INSERT INTO schema_migrations (name, checksum) VALUES ($1, $2)",
          [file, checksum],
        );
        await client.query("COMMIT");
      } catch (err) {
        await client.query("ROLLBACK");
        throw new Error(`Migration ${file} failed and rolled back: ${err instanceof Error ? err.message : String(err)}`);
      }
      applied.push(file);
      logger.info(`[DB] Applied migration ${file}`);
    }
    return applied;
  } finally {
    await client.query("SELECT pg_advisory_unlock($1)", [MIGRATION_LOCK_ID]).catch(() => undefined);
    client.release();
  }
}
