-- Migrate ledger_events uniqueness from (tx_hash, event_type) to
-- (tx_hash, event_index) so multiple same-typed events in one transaction
-- are preserved (#923).
--
-- Step 1: Add event_index column with a default that preserves existing
-- deduplication semantics — within each tx_hash, rows are numbered 0..N
-- by their insertion order (ctid).
ALTER TABLE ledger_events
  ADD COLUMN IF NOT EXISTS event_index INT NOT NULL DEFAULT 0;

-- Step 2: Assign unique event_index values to existing rows that share a
-- tx_hash, so the new unique constraint can be created without conflicts.
-- Rows that already have event_index = 0 (the default) get renumbered
-- only when there are duplicates within the same tx_hash.
WITH numbered AS (
  SELECT ctid,
         ROW_NUMBER() OVER (PARTITION BY tx_hash ORDER BY ctid) - 1 AS idx
  FROM ledger_events
)
UPDATE ledger_events e
SET event_index = n.idx
FROM numbered n
WHERE e.ctid = n.ctid
  AND n.idx > 0;

-- Step 3: Drop the old unique constraint/index and create the new one.
DROP INDEX IF EXISTS ledger_events_tx_hash_event_type_key;

CREATE UNIQUE INDEX IF NOT EXISTS ledger_events_tx_hash_event_index_key
  ON ledger_events (tx_hash, event_index);
