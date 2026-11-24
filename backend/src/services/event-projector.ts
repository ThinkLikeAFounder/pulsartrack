import prisma from "../db/prisma";
import { logger } from "../lib/logger";

type EventRecord = Record<string, unknown>;

function record(value: unknown): EventRecord {
  return value && typeof value === "object" ? (value as EventRecord) : {};
}

function text(value: unknown, fallback = "") {
  return typeof value === "string"
    ? value
    : value == null
      ? fallback
      : String(value);
}

function integer(value: unknown, fallback = 0n) {
  try {
    return BigInt(value as string | number | bigint);
  } catch {
    return fallback;
  }
}

/** Projects known Soroban events into queryable domain tables. Unknown events remain in ledger_events. */
export async function projectEvent(event: {
  event_type: string;
  event_data: unknown;
  tx_hash: string;
  ledger_sequence: number;
}) {
  const type = event.event_type.toLowerCase().replace(/[^a-z0-9]/g, "_");
  const data = record(event.event_data);
  try {
    if (
      type.includes("campaign") &&
      (type.includes("create") || type.includes("launch"))
    ) {
      const campaignId = integer(
        data.campaign_id ?? data.campaignId ?? data.id,
      );
      if (campaignId === 0n) return;
      await prisma.campaign.upsert({
        where: { campaignId },
        create: {
          campaignId,
          advertiser: text(data.advertiser ?? data.owner, "unknown"),
          title: text(data.title, "Untitled campaign"),
          contentId: text(data.content_id ?? data.contentId, ""),
          budgetStroops: integer(data.budget),
          txHash: event.tx_hash,
          ledgerSequence: BigInt(event.ledger_sequence),
        },
        update: {
          txHash: event.tx_hash,
          ledgerSequence: BigInt(event.ledger_sequence),
        },
      });
    } else if (
      type.includes("auction") &&
      (type.includes("create") || type.includes("open"))
    ) {
      const auctionId = integer(data.auction_id ?? data.auctionId ?? data.id);
      if (auctionId === 0n) return;
      await prisma.auction.upsert({
        where: { auctionId },
        create: {
          auctionId,
          publisher: text(data.publisher ?? data.owner, "unknown"),
          impressionSlot: text(data.impression_slot ?? data.slot),
          endTime: new Date(Date.now() + 86_400_000),
          txHash: event.tx_hash,
        },
        update: { txHash: event.tx_hash },
      });
    } else if (
      type.includes("publisher") &&
      (type.includes("register") || type.includes("create"))
    ) {
      const address = text(data.publisher ?? data.owner ?? data.address);
      if (!address) return;
      await prisma.publisher.upsert({
        where: { address },
        create: {
          address,
          displayName: text(data.display_name ?? data.displayName, address),
        },
        update: { lastActivity: new Date() },
      });
    } else if (
      type.includes("proposal") &&
      (type.includes("create") || type.includes("submit"))
    ) {
      const proposalId = integer(
        data.proposal_id ?? data.proposalId ?? data.id,
      );
      if (proposalId === 0n) return;
      await prisma.governanceProposal.upsert({
        where: { proposalId },
        create: {
          proposalId,
          proposer: text(data.proposer ?? data.owner, "unknown"),
          title: text(data.title),
          description: text(data.description),
          votingEndsAt: new Date(Date.now() + 7 * 86_400_000),
        },
        update: {},
      });
    } else if (type.includes("impression") || type.includes("view")) {
      const campaignId = integer(data.campaign_id ?? data.campaignId);
      const publisher = text(data.publisher ?? data.publisher_address);
      if (campaignId === 0n || !publisher) return;
      await prisma.impression.create({
        data: {
          campaignId,
          publisher,
          payoutStroops: integer(data.payout ?? data.payout_stroops),
          verified: Boolean(data.verified),
          ledgerSequence: BigInt(event.ledger_sequence),
        },
      });
    }
  } catch (error) {
    logger.warn(
      { error, eventType: event.event_type, txHash: event.tx_hash },
      "Failed to project indexed event",
    );
  }
}
