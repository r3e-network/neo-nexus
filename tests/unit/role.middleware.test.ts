import { describe, expect, it, vi } from "vitest";
import { createMockRequest, createMockResponse } from "../setup";
import { requireAdmin, requireAdminForUnsafeMethods } from "../../src/api/middleware/roles";

describe("role middleware", () => {
  it("blocks viewer users from admin-only handlers", () => {
    const req = createMockRequest({
      user: { id: "viewer-1", username: "viewer", role: "viewer" },
    });
    const res = createMockResponse();
    const next = vi.fn();

    requireAdmin(req as never, res as never, next);

    expect(res.status).toHaveBeenCalledWith(403);
    expect(res.jsonData).toEqual(expect.objectContaining({
      code: "ADMIN_REQUIRED",
      error: "Admin access required",
    }));
    expect(next).not.toHaveBeenCalled();
  });

  it("allows viewer users through safe read-only methods", () => {
    const req = createMockRequest({
      method: "GET",
      user: { id: "viewer-1", username: "viewer", role: "viewer" },
    });
    const res = createMockResponse();
    const next = vi.fn();

    requireAdminForUnsafeMethods(req as never, res as never, next);

    expect(next).toHaveBeenCalledOnce();
    expect(res.status).not.toHaveBeenCalled();
  });

  it("blocks viewer users from unsafe methods", () => {
    const req = createMockRequest({
      method: "POST",
      user: { id: "viewer-1", username: "viewer", role: "viewer" },
    });
    const res = createMockResponse();
    const next = vi.fn();

    requireAdminForUnsafeMethods(req as never, res as never, next);

    expect(res.status).toHaveBeenCalledWith(403);
    expect(res.jsonData).toEqual(expect.objectContaining({
      code: "ADMIN_REQUIRED",
    }));
    expect(next).not.toHaveBeenCalled();
  });
});
