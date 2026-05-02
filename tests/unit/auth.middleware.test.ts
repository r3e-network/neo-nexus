import { describe, expect, it, vi } from "vitest";
import jwt from "jsonwebtoken";
import { createMockRequest, createMockResponse } from "../setup";
import { createAuthMiddleware, getTokenExpiresInHours } from "../../src/api/middleware/auth";

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
    expect(res.jsonData).toEqual(expect.objectContaining({
      error: "Session expired or invalid",
      code: "SESSION_INVALID",
      suggestion: expect.any(String),
      status: 401,
    }));
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

  it("derives database session lifetime from the jwt exp claim", () => {
    const now = 1_800_000_000_000;
    vi.mocked(jwt.decode).mockReturnValue({
      userId: "test-user-id",
      username: "admin",
      exp: Math.floor((now + 2 * 60 * 60 * 1000) / 1000),
    });

    expect(getTokenExpiresInHours("valid-token", now)).toBe(2);
  });
});
