import prisma from '../prisma';
import { Prisma } from '@prisma/client';

export const DEFAULT_BID_LIMIT = 20;
export const MAX_BID_LIMIT = 100;

function normalizeBidLimit(limit = DEFAULT_BID_LIMIT): number {
  if (!Number.isFinite(limit) || limit <= 0) {
    return DEFAULT_BID_LIMIT;
  }
  return Math.min(Math.floor(limit), MAX_BID_LIMIT);
}

export async function findByAuction(auctionId: bigint, limit = DEFAULT_BID_LIMIT) {
  return prisma.bid.findMany({
    where: { auctionId },
    orderBy: { amountStroops: 'desc' },
    take: normalizeBidLimit(limit),
  });
}

export async function findByBidder(bidder: string, limit = DEFAULT_BID_LIMIT) {
  return prisma.bid.findMany({
    where: { bidder },
    orderBy: { timestamp: 'desc' },
    take: normalizeBidLimit(limit),
  });
}

export async function create(data: Prisma.BidCreateInput) {
  return prisma.bid.create({ data });
}

