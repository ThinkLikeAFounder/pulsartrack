import pool from '../../config/database';

interface LedgerEventRow {
  id: string;
  ledger_sequence: number;
  tx_hash: string;
  contract_id: string | null;
  event_type: string;
  event_data: unknown;
  indexed_at: Date;
}

export async function findByContract(contractId: string, limit = 50): Promise<LedgerEventRow[]> {
  const { rows } = await pool.query(
    `SELECT * FROM ledger_events
     WHERE contract_id = $1
     ORDER BY ledger_sequence DESC
     LIMIT $2`,
    [contractId, limit],
  );
  return rows;
}

export async function findByType(eventType: string, limit = 50): Promise<LedgerEventRow[]> {
  const { rows } = await pool.query(
    `SELECT * FROM ledger_events
     WHERE event_type = $1
     ORDER BY indexed_at DESC
     LIMIT $2`,
    [eventType, limit],
  );
  return rows;
}

export async function create(data: {
  ledger_sequence: number;
  tx_hash: string;
  contract_id?: string;
  event_type: string;
  event_data?: unknown;
}): Promise<LedgerEventRow | null> {
  const { rows } = await pool.query(
    `INSERT INTO ledger_events (ledger_sequence, tx_hash, contract_id, event_type, event_data)
     VALUES ($1, $2, $3, $4, $5)
     ON CONFLICT (tx_hash, event_type) DO NOTHING
     RETURNING *`,
    [data.ledger_sequence, data.tx_hash, data.contract_id ?? null, data.event_type, data.event_data ? JSON.stringify(data.event_data) : null],
  );
  return rows[0] ?? null;
}

export async function getLatestSequence(): Promise<bigint> {
  const { rows } = await pool.query(
    `SELECT COALESCE(MAX(ledger_sequence), 0) AS max_seq FROM ledger_events`,
  );
  return BigInt(rows[0]?.max_seq ?? 0);
}
