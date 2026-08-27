import { describe, it, expect, vi, beforeEach } from 'vitest';

const { mockFindMany, mockCreate, mockCount } = vi.hoisted(() => ({
  mockFindMany: vi.fn(),
  mockCreate: vi.fn(),
  mockCount: vi.fn(),
}));

vi.mock('../db/prisma', () => ({
  default: {
    impression: {
      findMany: mockFindMany,
      create: mockCreate,
      count: mockCount,
    },
  },
}));

import * as impressions from '../db/repositories/impressions';

describe('impressions repository', () => {
  beforeEach(() => vi.clearAllMocks());

  describe('findByCampaign', () => {
    it('returns impressions newest first with the default limit', async () => {
      mockFindMany.mockResolvedValue([{ impressionId: 1n }]);

      const result = await impressions.findByCampaign(1n);

      expect(result).toHaveLength(1);
      expect(mockFindMany).toHaveBeenCalledWith({
        where: { campaignId: 1n },
        orderBy: { timestamp: 'desc' },
        take: 50,
      });
    });

    it('honours an explicit limit', async () => {
      mockFindMany.mockResolvedValue([]);

      await impressions.findByCampaign(2n, 10);

      expect(mockFindMany).toHaveBeenCalledWith(
        expect.objectContaining({ take: 10 }),
      );
    });
  });

  describe('findByPublisher', () => {
    it('filters by publisher address, newest first', async () => {
      mockFindMany.mockResolvedValue([]);

      await impressions.findByPublisher('GPUBLISHER');

      expect(mockFindMany).toHaveBeenCalledWith({
        where: { publisher: 'GPUBLISHER' },
        orderBy: { timestamp: 'desc' },
        take: 50,
      });
    });

    it('honours an explicit limit', async () => {
      mockFindMany.mockResolvedValue([]);

      await impressions.findByPublisher('GPUBLISHER', 3);

      expect(mockFindMany).toHaveBeenCalledWith(
        expect.objectContaining({ take: 3 }),
      );
    });
  });

  describe('create', () => {
    it('passes the input through as prisma data', async () => {
      const input = { impressionId: 1n, campaignId: 1n } as never;
      mockCreate.mockResolvedValue(input);

      await impressions.create(input);

      expect(mockCreate).toHaveBeenCalledWith({ data: input });
    });
  });

  describe('countByCampaign', () => {
    it('counts the impressions recorded for a campaign', async () => {
      mockCount.mockResolvedValue(42);

      const result = await impressions.countByCampaign(9n);

      expect(result).toBe(42);
      expect(mockCount).toHaveBeenCalledWith({ where: { campaignId: 9n } });
    });
  });
});
