import { describe, it, expect, vi, beforeAll, beforeEach } from "vitest";
import type { Express, Request, Response } from "express";
import type supertest from "supertest";
import { createJwt } from "../lib/jwt";

// Mock logger to suppress output
vi.mock("../lib/logger", () => ({
  logger: { warn: vi.fn(), error: vi.fn(), info: vi.fn() },
}));

// Mock Redis to avoid real connections
vi.mock("../config/redis", () => ({
  default: {
    get: vi.fn().mockResolvedValue(null),
    set: vi.fn().mockResolvedValue("OK"),
    del: vi.fn().mockResolvedValue(1),
    incr: vi.fn().mockResolvedValue(1),
    expire: vi.fn().mockResolvedValue(1),
    disconnect: vi.fn().mockResolvedValue(undefined),
  },
}));

// Mock database to avoid real connections
vi.mock("../config/database", () => ({
  query: vi.fn().mockResolvedValue({ rows: [] }),
  pool: {
    query: vi.fn().mockResolvedValue({ rows: [] }),
    end: vi.fn().mockResolvedValue(undefined),
  },
}));

// Mock Prisma client
vi.mock("../db/prisma", () => ({
  default: {
    campaign: {
      create: vi.fn(),
      findMany: vi.fn(),
      count: vi.fn(),
      aggregate: vi.fn(),
    },
  },
}));

// Mock Soroban client
vi.mock("../services/soroban-client", () => ({
  callReadOnly: vi.fn().mockResolvedValue(0),
}));

describe("POST /api/campaigns (stellarAddress validation)", () => {
  let app: Express;
  let request: typeof supertest;

  beforeAll(async () => {
    const appModule = await import("../app");
    app = appModule.default;
    request = (await import("supertest")).default;
  });

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should return 401 when Authorization header is missing", async () => {
    const res = await request(app).post("/api/campaigns").send({
      title: "Test Campaign",
      contentId: "content-123",
      budgetStroops: 1000000,
      dailyBudgetStroops: 100000,
    });

    expect(res.status).toBe(401);
    expect(res.body.error).toBeDefined();
  });

  it("should return 401 when token is invalid", async () => {
    const res = await request(app)
      .post("/api/campaigns")
      .set("Authorization", "Bearer invalid.token.here")
      .send({
        title: "Test Campaign",
        contentId: "content-123",
        budgetStroops: 1000000,
        dailyBudgetStroops: 100000,
      });

    expect(res.status).toBe(401);
    expect(res.body.error).toBeDefined();
  });

  it("should return 401 when stellarAddress is somehow undefined despite passing requireAuth", async () => {
    // This test verifies the getAuthedAddress helper's defensive check
    // In practice, requireAuth should set stellarAddress, but the helper provides type safety
    const { getAuthedAddress } = await import("../middleware/auth");
    const mockReq = { stellarAddress: undefined } as unknown as Request;
    const mockRes = { status: vi.fn().mockReturnThis(), json: vi.fn() } as unknown as Response;

    const address = getAuthedAddress(mockReq, mockRes);

    expect(address).toBeNull();
    expect(mockRes.status).toHaveBeenCalledWith(401);
    expect(mockRes.json).toHaveBeenCalledWith({
      error: "Authentication required",
    });
  });

  it("should accept request with valid token and stellarAddress", async () => {
    const prisma = (await import("../db/prisma")).default;
    vi.mocked(prisma.campaign.create).mockResolvedValue({
      id: "test-uuid-123",
      campaignId: BigInt(1),
      advertiser: "GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N",
      title: "Test Campaign",
      contentId: "content-123",
      budgetStroops: BigInt(1000000),
      dailyBudgetStroops: BigInt(100000),
      spentStroops: BigInt(0),
      impressions: BigInt(0),
      clicks: BigInt(0),
      status: "Active",
      createdAt: new Date(),
      expiresAt: null,
      txHash: null,
      ledgerSequence: null,
    });

    const token = createJwt({
      sub: "GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N",
    });
    const res = await request(app)
      .post("/api/campaigns")
      .set("Authorization", `Bearer ${token}`)
      .send({
        title: "Test Campaign",
        contentId: "content-123",
        budgetStroops: 1000000,
        dailyBudgetStroops: 100000,
      });

    // May fail with rate limit or other errors, but should not be 401
    expect(res.status).not.toBe(401);
  });
});
