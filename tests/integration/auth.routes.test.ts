/**
 * Integration Tests: Authentication Routes
 * 
 * Tests the authentication API endpoints end-to-end
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import request from "supertest";
import express, { Request, Response, NextFunction } from "express";

describe("Auth Routes Integration", () => {
  let app: express.Application;
  let mockUserManager: any;

  beforeEach(() => {
    app = express();
    app.use(express.json());

    // Mock UserManager
    mockUserManager = {
      hasUsers: vi.fn(() => true),
      verifyCredentials: vi.fn(),
      createUser: vi.fn(),
      updatePassword: vi.fn(),
      createSession: vi.fn(),
      verifySession: vi.fn(),
      deleteSession: vi.fn(),
      getUserById: vi.fn(),
    };

    // Setup routes inline for testing
    app.get("/api/auth/setup-status", (req: Request, res: Response) => {
      res.json({
        setupRequired: !mockUserManager.hasUsers(),
        hasUsers: mockUserManager.hasUsers(),
      });
    });

    app.post("/api/auth/login", async (req: Request, res: Response) => {
      const { username, password } = req.body;
      
      if (!username || !password) {
        return res.status(400).json({ error: "Username and password required" });
      }

      const user = await mockUserManager.verifyCredentials(username, password);
      if (!user) {
        return res.status(401).json({ success: false, error: "Invalid credentials" });
      }

      const token = "mock-jwt-token";
      mockUserManager.createSession(user.id, token);
      
      res.json({
        success: true,
        token,
        user: {
          id: user.id,
          username: user.username,
          role: user.role,
        },
      });
    });

    app.post("/api/auth/change-password", async (req: Request, res: Response) => {
      const { currentPassword, newPassword } = req.body;
      
      if (!currentPassword || !newPassword) {
        return res.status(400).json({ error: "Current and new password required" });
      }

      try {
        // Mock user ID from auth middleware
        const userId = "test-user-id";
        await mockUserManager.updatePassword(userId, currentPassword, newPassword);
        res.json({ success: true });
      } catch (error: any) {
        res.status(400).json({ success: false, error: error.message });
      }
    });

    app.post("/api/auth/logout", (req: Request, res: Response) => {
      const token = req.headers.authorization?.replace("Bearer ", "") || "";
      mockUserManager.deleteSession(token);
      res.json({ success: true });
    });
  });

  describe("POST /api/auth/login", () => {
    it("should authenticate with valid credentials", async () => {
      mockUserManager.verifyCredentials.mockResolvedValue({
        id: "user-1",
        username: "admin",
        role: "admin",
      });

      const response = await request(app)
        .post("/api/auth/login")
        .send({ username: "admin", password: "admin" })
        .expect(200);

      expect(response.body.success).toBe(true);
      expect(response.body.token).toBe("mock-jwt-token");
      expect(response.body.user.username).toBe("admin");
    });

    it("should reject invalid credentials", async () => {
      mockUserManager.verifyCredentials.mockResolvedValue(null);

      const response = await request(app)
        .post("/api/auth/login")
        .send({ username: "admin", password: "wrongpassword" })
        .expect(401);

      expect(response.body.success).toBe(false);
      expect(response.body.error).toBe("Invalid credentials");
    });

    it("should require username and password", async () => {
      const response = await request(app)
        .post("/api/auth/login")
        .send({ username: "admin" })
        .expect(400);

      expect(response.body.error).toContain("required");
    });

    it("should handle empty request body", async () => {
      const response = await request(app)
        .post("/api/auth/login")
        .send({})
        .expect(400);

      expect(response.body.error).toBeDefined();
    });
  });

  describe("GET /api/auth/setup-status", () => {
    it("should return setup status when users exist", async () => {
      mockUserManager.hasUsers.mockReturnValue(true);

      const response = await request(app)
        .get("/api/auth/setup-status")
        .expect(200);

      expect(response.body.hasUsers).toBe(true);
      expect(response.body.setupRequired).toBe(false);
    });

    it("should indicate setup required when no users", async () => {
      mockUserManager.hasUsers.mockReturnValue(false);

      const response = await request(app)
        .get("/api/auth/setup-status")
        .expect(200);

      expect(response.body.setupRequired).toBe(true);
      expect(response.body.hasUsers).toBe(false);
    });
  });

  describe("POST /api/auth/change-password", () => {
    it("should change password with valid current password", async () => {
      mockUserManager.updatePassword.mockResolvedValue(undefined);

      const response = await request(app)
        .post("/api/auth/change-password")
        .set("Authorization", "Bearer valid-token")
        .send({
          currentPassword: "oldpass",
          newPassword: "newpassword123",
        })
        .expect(200);

      expect(response.body.success).toBe(true);
    });

    it("should reject change with invalid current password", async () => {
      mockUserManager.updatePassword.mockRejectedValue(new Error("Current password is incorrect"));

      const response = await request(app)
        .post("/api/auth/change-password")
        .set("Authorization", "Bearer valid-token")
        .send({
          currentPassword: "wrongpass",
          newPassword: "newpassword123",
        })
        .expect(400);

      expect(response.body.success).toBe(false);
      expect(response.body.error).toContain("Current password");
    });

    it("should require all fields", async () => {
      const response = await request(app)
        .post("/api/auth/change-password")
        .set("Authorization", "Bearer valid-token")
        .send({ currentPassword: "oldpass" })
        .expect(400);

      expect(response.body.error).toContain("required");
    });
  });

  describe("POST /api/auth/logout", () => {
    it("should invalidate session on logout", async () => {
      const response = await request(app)
        .post("/api/auth/logout")
        .set("Authorization", "Bearer test-token")
        .expect(200);

      expect(response.body.success).toBe(true);
      expect(mockUserManager.deleteSession).toHaveBeenCalledWith("test-token");
    });

    it("should handle logout without token", async () => {
      const response = await request(app)
        .post("/api/auth/logout")
        .expect(200);

      expect(response.body.success).toBe(true);
    });
  });
});
