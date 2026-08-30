import pool from '../../config/database';

interface SubscriptionRow {
  id: string;
  subscriber: string;
  tier: string;
  is_annual: boolean;
  amount_paid_stroops: number;
  started_at: Date;
  expires_at: Date;
  auto_renew: boolean;
  tx_hash: string | null;
}

export async function findBySubscriber(subscriber: string): Promise<SubscriptionRow[]> {
  const { rows } = await pool.query(
    `SELECT * FROM subscriptions
     WHERE subscriber = $1
     ORDER BY started_at DESC`,
    [subscriber],
  );
  return rows;
}

export async function findActive(subscriber: string): Promise<SubscriptionRow | null> {
  const { rows } = await pool.query(
    `SELECT * FROM subscriptions
     WHERE subscriber = $1 AND expires_at > NOW()
     ORDER BY expires_at DESC
     LIMIT 1`,
    [subscriber],
  );
  return rows[0] ?? null;
}

export async function create(data: {
  subscriber: string;
  tier: string;
  is_annual?: boolean;
  amount_paid_stroops: number;
  started_at?: Date;
  expires_at: Date;
  auto_renew?: boolean;
  tx_hash?: string;
}): Promise<SubscriptionRow> {
  const { rows } = await pool.query(
    `INSERT INTO subscriptions (subscriber, tier, is_annual, amount_paid_stroops, started_at, expires_at, auto_renew, tx_hash)
     VALUES ($1, $2, $3, $4, COALESCE($5, NOW()), $6, $7, $8)
     RETURNING *`,
    [
      data.subscriber,
      data.tier,
      data.is_annual ?? false,
      data.amount_paid_stroops,
      data.started_at ?? null,
      data.expires_at,
      data.auto_renew ?? true,
      data.tx_hash ?? null,
    ],
  );
  return rows[0];
}
