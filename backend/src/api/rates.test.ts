import { describe, it, expect, vi } from "vitest";
import request from "supertest";
import app from "../app";

vi.mock("../services/horizon", () => ({
  getFeeStats: vi.fn().mockResolvedValue({
    lastLedger: 123,
    feeCharged: { min: "100", mode: "100", p10: "100", p20: "100", p50: "100", p90: "100", p99: "100" },
  }),
}));

describe("GET /api/rates", () => {
  it("includes a Cache-Control header", async () => {
    const response = await request(app).get("/api/rates");

    expect(response.status).toBe(200);
    expect(response.headers["cache-control"]).toBe("public, max-age=60");
  });
});
