import pool from '../../config/database';

export const DEFAULT_SUBSCRIPTION_HISTORY_LIMIT = 50;
export const MAX_SUBSCRIPTION_HISTORY_LIMIT = 100;

interface SubscriptionRow {
  id: string;
  subscriber: string;
  tier: string;
  is_annual: boolean;
  amount_paid_stroops: bigint;
  started_at: Date;
  expires_at: Date;
  auto_renew: boolean;
  tx_hash: string | null;
}

// node-postgres returns BIGINT (OID 20) columns as strings to avoid silent
// precision loss, so the raw row shape differs from SubscriptionRow.
interface SubscriptionDbRow extends Omit<SubscriptionRow, 'amount_paid_stroops'> {
  amount_paid_stroops: string;
}

function toSubscriptionRow(row: SubscriptionDbRow): SubscriptionRow {
  return {
    ...row,
    amount_paid_stroops: row.amount_paid_stroops != null ? BigInt(row.amount_paid_stroops) : row.amount_paid_stroops,
  };
}

function normalizeSubscriptionLimit(take = DEFAULT_SUBSCRIPTION_HISTORY_LIMIT): number {
  if (!Number.isFinite(take) || take <= 0) {
    return DEFAULT_SUBSCRIPTION_HISTORY_LIMIT;
  }
  return Math.min(Math.floor(take), MAX_SUBSCRIPTION_HISTORY_LIMIT);
}

export async function findBySubscriber(
  subscriber: string,
  take = DEFAULT_SUBSCRIPTION_HISTORY_LIMIT,
): Promise<SubscriptionRow[]> {
  const limit = normalizeSubscriptionLimit(take);
  const { rows } = await pool.query<SubscriptionDbRow>(
    `SELECT * FROM subscriptions
     WHERE subscriber = $1
     ORDER BY started_at DESC
     LIMIT $2`,
    [subscriber, limit],
  );
  return rows.map(toSubscriptionRow);
}

export async function findActive(subscriber: string): Promise<SubscriptionRow | null> {
  const { rows } = await pool.query<SubscriptionDbRow>(
    `SELECT * FROM subscriptions
     WHERE subscriber = $1 AND expires_at > NOW()
     ORDER BY expires_at DESC
     LIMIT 1`,
    [subscriber],
  );
  return rows[0] ? toSubscriptionRow(rows[0]) : null;
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
  const { rows } = await pool.query<SubscriptionDbRow>(
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
  return toSubscriptionRow(rows[0]);
}