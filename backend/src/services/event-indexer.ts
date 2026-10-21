import { getServer } from './soroban-client';
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

    // getEvents uses a discriminated union: ledger-range mode requires a
    // concrete number for startLedger (not undefined). Fall back to ledger 1
    // on the first run when no events have been indexed yet.
    const from: number = startSeq > 0 ? startSeq + 1 : 1;
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
        // SDK BaseEventResponse exposes txHash, not transactionHash
        const txHash = event.txHash || '';
        const ledgerSeq = event.ledger || 0;
        // contractId is typed as Contract (SDK class) — convert to string
        const contractId = event.contractId ? event.contractId.toString() : '';
        const eventType = event.topic?.[0] || 'unknown';
        const eventData = event.value || null;

        // Extract the stable event_index from the event's unique id field.
        // The id format is "{TOID}-{event_index}" where event_index is a
        // 10-character zero-padded number representing the event's position
        // within its transaction. This is stable across polls for the same
        // event, unlike the loop counter i which resets each poll (#1094).
        const eventId = (event as any).id || '';
        const eventIndexStr = eventId.split('-')[1] || '0';
        const eventIndex = parseInt(eventIndexStr, 10) || 0;

        const result = await ledgerEventsRepo.create({
          ledger_sequence: ledgerSeq,
          tx_hash: txHash,
          event_index: eventIndex,
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
