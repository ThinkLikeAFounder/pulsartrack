import { describe, it, expect, vi } from "vitest";
import { validate } from "./validate";
import { Request, Response, NextFunction } from "express";

describe("validate middleware", () => {
  const mockReq = (data: any = {}) =>
    ({
      params: data.params || {},
      query: data.query || {},
      body: data.body || {},
    }) as unknown as Request;

  const mockRes = () => {
    const res = {} as Response;
    res.status = vi.fn().mockReturnValue(res);
    res.json = vi.fn().mockReturnValue(res);
    return res;
  };

  const mockNext = vi.fn() as NextFunction;

  it("should pass validation for valid stellar address", () => {
    const middleware = validate({
      body: {
        address: { type: "stellar_address", required: true },
      },
    });

    const req = mockReq({
      body: {
        address: "GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFSHONUCEOASW7QC7OX2H",
      },
    });
    const res = mockRes();
    const next = vi.fn();

    middleware(req, res, next);

    expect(next).toHaveBeenCalled();
    expect(res.status).not.toHaveBeenCalled();
  });

  it("should reject invalid stellar address", () => {
    const middleware = validate({
      body: {
        address: { type: "stellar_address", required: true },
      },
    });

    const req = mockReq({
      body: { address: "invalid-address" },
    });
    const res = mockRes();
    const next = vi.fn();

    middleware(req, res, next);

    expect(next).not.toHaveBeenCalled();
    expect(res.status).toHaveBeenCalledWith(400);
    expect(res.json).toHaveBeenCalledWith(
      expect.objectContaining({
        error: "Validation failed",
        details: expect.arrayContaining([
          expect.objectContaining({
            field: "address",
            message: expect.stringContaining("valid Stellar public key"),
          }),
        ]),
      }),
    );
  });

  it("should reject missing required field", () => {
    const middleware = validate({
      body: {
        name: { type: "string", required: true },
      },
    });

    const req = mockReq({ body: {} });
    const res = mockRes();
    const next = vi.fn();

    middleware(req, res, next);

    expect(next).not.toHaveBeenCalled();
    expect(res.status).toHaveBeenCalledWith(400);
  });

  it("should validate number types with min/max", () => {
    const middleware = validate({
      query: {
        limit: { type: "number", integer: true, min: 1, max: 100 },
      },
    });

    const req = mockReq({ query: { limit: "50" } });
    const res = mockRes();
    const next = vi.fn();

    middleware(req, res, next);

    expect(next).toHaveBeenCalled();
  });

  it("should reject numbers outside min/max range", () => {
    const middleware = validate({
      query: {
        limit: { type: "number", integer: true, min: 1, max: 100 },
      },
    });

    const req = mockReq({ query: { limit: "150" } });
    const res = mockRes();
    const next = vi.fn();

    middleware(req, res, next);

    expect(next).not.toHaveBeenCalled();
    expect(res.status).toHaveBeenCalledWith(400);
  });

  it("should reject non-integer when integer is required", () => {
    const middleware = validate({
      body: {
        count: { type: "number", integer: true, required: true },
      },
    });

    const req = mockReq({ body: { count: 3.14 } });
    const res = mockRes();
    const next = vi.fn();

    middleware(req, res, next);

    expect(next).not.toHaveBeenCalled();
    expect(res.status).toHaveBeenCalledWith(400);
  });

  it("should validate string length constraints", () => {
    const middleware = validate({
      body: {
        name: { type: "string", minLength: 3, maxLength: 50 },
      },
    });

    const req = mockReq({ body: { name: "Valid Name" } });
    const res = mockRes();
    const next = vi.fn();

    middleware(req, res, next);

    expect(next).toHaveBeenCalled();
  });

  it("should reject string shorter than minLength", () => {
    const middleware = validate({
      body: {
        name: { type: "string", minLength: 5, required: true },
      },
    });

    const req = mockReq({ body: { name: "Hi" } });
    const res = mockRes();
    const next = vi.fn();

    middleware(req, res, next);

    expect(next).not.toHaveBeenCalled();
    expect(res.status).toHaveBeenCalledWith(400);
  });

  it("should validate URL format", () => {
    const middleware = validate({
      body: {
        website: { type: "string", format: "url", required: true },
      },
    });

    const req = mockReq({ body: { website: "https://example.com" } });
    const res = mockRes();
    const next = vi.fn();

    middleware(req, res, next);

    expect(next).toHaveBeenCalled();
  });

  it("should reject invalid URL format", () => {
    const middleware = validate({
      body: {
        website: { type: "string", format: "url", required: true },
      },
    });

    const req = mockReq({ body: { website: "not-a-url" } });
    const res = mockRes();
    const next = vi.fn();

    middleware(req, res, next);

    expect(next).not.toHaveBeenCalled();
    expect(res.status).toHaveBeenCalledWith(400);
  });

  it("should allow optional fields to be omitted", () => {
    const middleware = validate({
      body: {
        optional: { type: "string", required: false },
      },
    });

    const req = mockReq({ body: {} });
    const res = mockRes();
    const next = vi.fn();

    middleware(req, res, next);

    expect(next).toHaveBeenCalled();
  });
});
