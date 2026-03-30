import { describe, expect, it, vi } from "vitest";
import { respondWithApiError } from "../../src/api/respond";
import { ApiError } from "../../src/api/errors";

function mockResponse() {
  const json = vi.fn();
  const status = vi.fn(() => ({ json }));
  return { status, json, raw: { status, json } as unknown };
}

describe("respondWithApiError", () => {
  it("serializes an ApiError with all fields", () => {
    const res = mockResponse();
    const err = new ApiError("NODE_NOT_FOUND", "Node not found", "Check the ID.", 404);

    respondWithApiError(res.raw as never, err);

    expect(res.status).toHaveBeenCalledWith(404);
    expect(res.json).toHaveBeenCalledWith({
      error: "Node not found",
      code: "NODE_NOT_FOUND",
      suggestion: "Check the ID.",
      status: 404,
    });
  });

  it("falls back to 500 for plain Error", () => {
    const res = mockResponse();
    respondWithApiError(res.raw as never, new Error("something broke"));

    expect(res.status).toHaveBeenCalledWith(500);
    expect(res.json).toHaveBeenCalledWith(
      expect.objectContaining({ error: "something broke", status: 500 }),
    );
  });

  it("falls back to 500 for non-Error values", () => {
    const res = mockResponse();
    respondWithApiError(res.raw as never, "string error");

    expect(res.status).toHaveBeenCalledWith(500);
    expect(res.json).toHaveBeenCalledWith(
      expect.objectContaining({ error: "string error", status: 500 }),
    );
  });

  it("maps plain Error 'not found' messages to 404", () => {
    const res = mockResponse();
    respondWithApiError(res.raw as never, new Error("Node xyz not found"));

    expect(res.status).toHaveBeenCalledWith(404);
  });

  it("maps plain Error validation messages to 400", () => {
    const res = mockResponse();
    respondWithApiError(res.raw as never, new Error("Cannot update while running"));

    expect(res.status).toHaveBeenCalledWith(400);
  });
});
