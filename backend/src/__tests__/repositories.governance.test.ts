import { describe, it, expect, vi, beforeEach } from 'vitest';

const { mockFindMany, mockFindUnique, mockCreate, mockUpdate } = vi.hoisted(() => ({
  mockFindMany: vi.fn(),
  mockFindUnique: vi.fn(),
  mockCreate: vi.fn(),
  mockUpdate: vi.fn(),
}));

vi.mock('../db/prisma', () => ({
  default: {
    governanceProposal: {
      findMany: mockFindMany,
      findUnique: mockFindUnique,
      create: mockCreate,
      update: mockUpdate,
    },
  },
}));

import * as governance from '../db/repositories/governance';

describe('governance repository', () => {
  beforeEach(() => vi.clearAllMocks());

  describe('findMany', () => {
    it('returns proposals newest first with the default limit', async () => {
      mockFindMany.mockResolvedValue([{ proposalId: 1n }]);

      const result = await governance.findMany();

      expect(result).toHaveLength(1);
      expect(mockFindMany).toHaveBeenCalledWith({
        where: undefined,
        orderBy: { createdAt: 'desc' },
        take: 20,
      });
    });

    it('filters by status when one is supplied', async () => {
      mockFindMany.mockResolvedValue([]);

      await governance.findMany({ status: 'Active' }, 5);

      expect(mockFindMany).toHaveBeenCalledWith({
        where: { status: 'Active' },
        orderBy: { createdAt: 'desc' },
        take: 5,
      });
    });

    it('omits the where clause when the filter has no status', async () => {
      mockFindMany.mockResolvedValue([]);

      await governance.findMany({});

      expect(mockFindMany).toHaveBeenCalledWith(
        expect.objectContaining({ where: undefined }),
      );
    });
  });

  describe('findByProposalId', () => {
    it('looks the proposal up by its id', async () => {
      mockFindUnique.mockResolvedValue({ proposalId: 7n });

      const result = await governance.findByProposalId(7n);

      expect(result).toEqual({ proposalId: 7n });
      expect(mockFindUnique).toHaveBeenCalledWith({ where: { proposalId: 7n } });
    });

    it('returns null when no proposal matches', async () => {
      mockFindUnique.mockResolvedValue(null);

      await expect(governance.findByProposalId(404n)).resolves.toBeNull();
    });
  });

  describe('create', () => {
    it('passes the input through as prisma data', async () => {
      const input = { proposalId: 1n, title: 'Raise the fee floor' } as never;
      mockCreate.mockResolvedValue(input);

      await governance.create(input);

      expect(mockCreate).toHaveBeenCalledWith({ data: input });
    });
  });

  describe('recordVote', () => {
    it.each([
      ['for', 'votesFor'],
      ['against', 'votesAgainst'],
      ['abstain', 'votesAbstain'],
    ] as const)('increments %s votes on the %s column', async (vote, field) => {
      mockUpdate.mockResolvedValue({ proposalId: 3n });

      await governance.recordVote(3n, vote);

      expect(mockUpdate).toHaveBeenCalledWith({
        where: { proposalId: 3n },
        data: { [field]: { increment: 1 } },
      });
    });
  });
});
