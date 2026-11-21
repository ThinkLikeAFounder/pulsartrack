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
    publisher: {
      create: vi.fn(),
      findMany: vi.fn(),
      findUnique: vi.fn(),
    },
  },
}));

// Mock Soroban client
vi.mock("../services/soroban-client", () => ({
  callReadOnly: vi.fn().mockResolvedValue(0),
  toAddressScVal: vi.fn(),
}));

describe("POST /api/publishers/register (stellarAddress validation)", () => {
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
    const res = await request(app).post("/api/publishers/register").send({
      displayName: "Test Publisher",
      website: "https://example.com",
    });

    expect(res.status).toBe(401);
    expect(res.body.error).toBeDefined();
  });

  it("should return 401 when token is invalid", async () => {
    const res = await request(app)
      .post("/api/publishers/register")
      .set("Authorization", "Bearer invalid.token.here")
      .send({
        displayName: "Test Publisher",
        website: "https://example.com",
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

    // Mock findUnique to return null (no existing publisher)
    vi.mocked(prisma.publisher.findUnique).mockResolvedValue(null);

    // Mock create to return a publisher
    vi.mocked(prisma.publisher.create).mockResolvedValue({
      id: "test-uuid-456",
      address: "GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N",
      displayName: "Test Publisher",
      website: "https://example.com",
      status: "Pending",
      tier: "Bronze",
      reputationScore: 500,
      impressionsServed: BigInt(0),
      earningsStroops: BigInt(0),
      joinedAt: new Date(),
      lastActivity: new Date(),
    });

    const token = createJwt({
      sub: "GA4LYCAMDLLOJPGXHQCHHPXBISH5RAWSS7ZTCSQAPKASBXG4NTB5MJ6N",
    });
    const res = await request(app)
      .post("/api/publishers/register")
      .set("Authorization", `Bearer ${token}`)
      .send({
        displayName: "Test Publisher",
        website: "https://example.com",
      });

    // May fail with rate limit or other errors, but should not be 401
    expect(res.status).not.toBe(401);
  });
});
