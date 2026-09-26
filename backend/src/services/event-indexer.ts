import { getServer } from './soroban-client';
import { stellarConfig } from '../config/stellar';
import * as ledgerEventsRepo from '../db/repositories/ledgerEvents';
import { logger } from '../lib/logger';

/**
 * Background event indexer that reads Soroban contract events from the RPC
 * and writes them to the `ledger_events` table via the repository (#922).
 *
 * Resumes from the highest ledger sequence already stored, so it is safe to
 * restart without re-indexing the entire chain.
 */

const POLL_INTERVAL_MS = parseInt(process.env.EVENT_POLL_INTERVAL_MS || '5000', 10);
const MAX_EVENTS_PER_POLL = parseInt(process.env.EVENT_MAX_PER_POLL || '200', 10);

let running = false;
let pollTimer: ReturnType<typeof setTimeout> | null = null;

export function startEventIndexer() {
  if (running) return;
  running = true;
  logger.info('[EventIndexer] Starting event indexer');
  poll().catch((err) => {
    logger.error({ err }, '[EventIndexer] Fatal error in indexer loop');
  });
}

export function stopEventIndexer() {
  running = false;
  if (pollTimer) {
    clearTimeout(pollTimer);
    pollTimer = null;
  }
  logger.info('[EventIndexer] Stopped');
}

async function poll() {
  if (!running) return;

  try {
    const startSeq = Number(await ledgerEventsRepo.getLatestSequence());
    const server = getServer();

    // getEvents returns contract events in a range; start from the next ledger
    const from = startSeq > 0 ? startSeq + 1 : undefined;
    const response = await server.getEvents({
      startLedger: from,
      limit: MAX_EVENTS_PER_POLL,
      filters: [],
    });

    const events = response.events ?? [];
    if (events.length > 0) {
      let written = 0;
      for (let i = 0; i < events.length; i++) {
        const event = events[i];
        const txHash = event.transactionHash || '';
        const ledgerSeq = event.ledger || 0;
        const contractId = event.contractId || '';
        const eventType = event.topic?.[0] || 'unknown';
        const eventData = event.value || null;

        const result = await ledgerEventsRepo.create({
          ledger_sequence: ledgerSeq,
          tx_hash: txHash,
          event_index: i,
          contract_id: contractId,
          event_type: typeof eventType === 'string' ? eventType : JSON.stringify(eventType),
          event_data: eventData,
        });

        if (result) written++;
      }
      logger.info(`[EventIndexer] Indexed ${written}/${events.length} events from ledger ${events[0]?.ledger ?? '?'}`);
    }
  } catch (err: any) {
    // Transient RPC errors are expected during network issues — log and
    // continue polling rather than crashing the server.
    logger.warn({ err: err.message }, '[EventIndexer] Poll error, will retry');
  }

  if (running) {
    pollTimer = setTimeout(poll, POLL_INTERVAL_MS);
  }
}
