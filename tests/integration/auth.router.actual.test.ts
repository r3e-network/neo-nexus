import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { createAuthRouter } from "../../src/api/routes/auth";

describe("Actual auth router protection", () => {
  let app: express.Application;
  let mockUserManager: {
    hasUsers: ReturnType<typeof vi.fn>;
    verifyCredentials: ReturnType<typeof vi.fn>;
    verifySession: ReturnType<typeof vi.fn>;
    isUsingDefaultPassword: ReturnType<typeof vi.fn>;
    createUser: ReturnType<typeof vi.fn>;
    createSession: ReturnType<typeof vi.fn>;
    deleteSession: ReturnType<typeof vi.fn>;
    updatePassword: ReturnType<typeof vi.fn>;
    getAllUsers: ReturnType<typeof vi.fn>;
    deleteUser: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());

    mockUserManager = {
      hasUsers: vi.fn(() => true),
      verifyCredentials: vi.fn(),
      verifySession: vi.fn(() => ({
        id: "test-user-id",
        username: "admin",
        role: "admin",
      })),
      isUsingDefaultPassword: vi.fn(() => true),
      createUser: vi.fn(),
      createSession: vi.fn(),
      deleteSession: vi.fn(),
      updatePassword: vi.fn(),
      getAllUsers: vi.fn(() => [{ id: "admin-1", username: "admin", role: "admin" }]),
      deleteUser: vi.fn(),
    };

    app.use("/api/auth", createAuthRouter(mockUserManager as never));
  });

  it("changes password when an authenticated user sends a valid bearer token", async () => {
    mockUserManager.updatePassword.mockResolvedValue(undefined);

    const response = await request(app)
      .put("/api/auth/password")
      .set("Authorization", "Bearer valid-token")
      .send({
        currentPassword: "admin",
        newPassword: "new-password-123",
      });

    expect(response.status).toBe(200);
    expect(response.body.message).toBe("Password updated successfully");
    expect(mockUserManager.updatePassword).toHaveBeenCalledWith(
      "test-user-id",
      "admin",
      "new-password-123",
    );
  });

  it("includes default-password state in the login response", async () => {
    mockUserManager.verifyCredentials.mockResolvedValue({
      id: "test-user-id",
      username: "admin",
      role: "admin",
    });
    mockUserManager.isUsingDefaultPassword.mockReturnValue(false);

    const response = await request(app)
      .post("/api/auth/login")
      .send({
        username: "admin",
        password: "admin12345",
      });

    expect(response.status).toBe(200);
    expect(response.body.user).toEqual({
      id: "test-user-id",
      username: "admin",
      role: "admin",
      usingDefaultPassword: false,
    });
  });

  it("registers a new user for an authenticated admin", async () => {
    mockUserManager.createUser.mockResolvedValue({
      id: "viewer-1",
      username: "viewer",
      role: "viewer",
    });

    const response = await request(app)
      .post("/api/auth/register")
      .set("Authorization", "Bearer valid-token")
      .send({
        username: "viewer",
        password: "viewer-password",
        role: "viewer",
      });

    expect(response.status).toBe(201);
    expect(response.body.user).toEqual({
      id: "viewer-1",
      username: "viewer",
      role: "viewer",
    });
  });

  it("lists users for an authenticated admin", async () => {
    const response = await request(app)
      .get("/api/auth/users")
      .set("Authorization", "Bearer valid-token");

    expect(response.status).toBe(200);
    expect(response.body.users).toEqual([{ id: "admin-1", username: "admin", role: "admin" }]);
  });

  it("returns actual default-password state on /me", async () => {
    mockUserManager.isUsingDefaultPassword.mockReturnValue(false);

    const response = await request(app)
      .get("/api/auth/me")
      .set("Authorization", "Bearer valid-token");

    expect(response.status).toBe(200);
    expect(response.body.user).toEqual({
      id: "test-user-id",
      username: "admin",
      role: "admin",
      usingDefaultPassword: false,
    });
  });
});
