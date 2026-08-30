import { Pool } from "pg";
import { logger } from "../lib/logger";

// Validate DB_PASSWORD in production
const password = process.env.DB_PASSWORD;
if (!password && process.env.NODE_ENV === "production") {
  throw new Error("DB_PASSWORD is required in production");
}
if (!password) {
  logger.warn("DB_PASSWORD not set — using empty password (development only)");
}

const pool = new Pool({
  host: process.env.DB_HOST || "localhost",
  port: parseInt(process.env.DB_PORT || "5432"),
  database: process.env.DB_NAME || "pulsartrack",
  user: process.env.DB_USER || "postgres",
  password: password || "",
  max: 20,
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
});

export default pool;

export async function checkDbConnection(): Promise<boolean> {
  try {
    const client = await pool.connect();
    await client.query("SELECT 1");
    client.release();
    return true;
  } catch {
    return false;
  }
}
