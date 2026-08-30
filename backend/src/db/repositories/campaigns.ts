import prisma from '../prisma';
import { Prisma } from '@prisma/client';

export async function findMany(filter?: { status?: string }, limit = 20) {
  return prisma.campaign.findMany({
    where: filter?.status ? { status: filter.status } : undefined,
    orderBy: { createdAt: 'desc' },
    take: limit,
  });
}

export async function findByCampaignId(campaignId: bigint) {
  return prisma.campaign.findUnique({ where: { campaignId } });
}

export async function getStats() {
  // Issue #371 — use Promise.allSettled so a single failing query (e.g. a
  // transient connection error or table lock on the expensive aggregate) does
  // not discard results from the queries that succeeded.
  const [totalResult, activeResult, aggResult] = await Promise.allSettled([
    prisma.campaign.count(),
    prisma.campaign.count({ where: { status: 'Active' } }),
    prisma.campaign.aggregate({
      _sum: { impressions: true, clicks: true, spentStroops: true },
    }),
  ]);

  return {
    totalCampaigns:
      totalResult.status === 'fulfilled' ? totalResult.value : null,
    activeCampaigns:
      activeResult.status === 'fulfilled' ? activeResult.value : null,
    totalImpressions:
      aggResult.status === 'fulfilled'
        ? Number(aggResult.value._sum.impressions ?? 0)
        : null,
    totalClicks:
      aggResult.status === 'fulfilled'
        ? Number(aggResult.value._sum.clicks ?? 0)
        : null,
    totalSpentStroops:
      aggResult.status === 'fulfilled'
        ? Number(aggResult.value._sum.spentStroops ?? 0)
        : null,
    // Expose which sub-queries failed so callers can show partial-data notices.
    _partial:
      [totalResult, activeResult, aggResult].some((r) => r.status === 'rejected')
        ? {
            total: totalResult.status === 'rejected',
            active: activeResult.status === 'rejected',
            aggregate: aggResult.status === 'rejected',
          }
        : undefined,
  };
}

export async function create(data: Prisma.CampaignCreateInput) {
  return prisma.campaign.create({ data });
}

export async function updateStatus(campaignId: bigint, status: string) {
  return prisma.campaign.update({
    where: { campaignId },
    data: { status },
  });
}
