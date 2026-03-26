import { describe, expect, it, vi } from "vitest";
import { createMockRequest, createMockResponse } from "../setup";
import { createAuthMiddleware } from "../../src/api/middleware/auth";

describe("createAuthMiddleware", () => {
  it("rejects a valid jwt when the session has been deleted or expired", () => {
    const verifySession = vi.fn(() => null);
    const middleware = createAuthMiddleware({ verifySession } as never);
    const req = createMockRequest({
      headers: { authorization: "Bearer valid-token" },
      user: undefined,
    });
    const res = createMockResponse();
    const next = vi.fn();

    middleware(req as never, res as never, next);

    expect(verifySession).toHaveBeenCalledWith("valid-token");
    expect(res.status).toHaveBeenCalledWith(401);
    expect(res.jsonData).toEqual({ error: "Session expired or invalid" });
    expect(next).not.toHaveBeenCalled();
  });

  it("attaches the session user when jwt and session are both valid", () => {
    const sessionUser = {
      id: "test-user-id",
      username: "admin",
      role: "admin",
    };
    const verifySession = vi.fn(() => sessionUser);
    const middleware = createAuthMiddleware({ verifySession } as never);
    const req = createMockRequest({
      headers: { authorization: "Bearer valid-token" },
      user: undefined,
    });
    const res = createMockResponse();
    const next = vi.fn();

    middleware(req as never, res as never, next);

    expect(verifySession).toHaveBeenCalledWith("valid-token");
    expect(req.user).toEqual(sessionUser);
    expect(next).toHaveBeenCalledOnce();
  });
});
