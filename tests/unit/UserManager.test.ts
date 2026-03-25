/**
 * Unit Tests: UserManager
 * 
 * Tests user management with mocked database
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { UserManager } from "../../src/core/UserManager";

// Create a mock database
function createMockDb() {
  const users: any[] = [];
  const sessions: any[] = [];

  return {
    prepare: vi.fn((sql: string) => {
      // User queries
      if (sql.includes("SELECT id FROM users WHERE username")) {
        return {
          get: vi.fn((username: string) => {
            return users.find(u => u.username === username) || undefined;
          }),
        };
      }

      if (sql.includes("SELECT * FROM users WHERE username")) {
        return {
          get: vi.fn((username: string) => {
            const user = users.find(u => u.username === username);
            if (user) {
              return {
                id: user.id,
                username: user.username,
                password_hash: user.password_hash,
                role: user.role,
                created_at: user.created_at,
                last_login: user.last_login || null,
              };
            }
            return undefined;
          }),
        };
      }

      if (sql.includes("SELECT * FROM users WHERE id")) {
        return {
          get: vi.fn((id: string) => {
            const user = users.find(u => u.id === id);
            return user ? {
              id: user.id,
              username: user.username,
              role: user.role,
              created_at: user.created_at,
              last_login: user.last_login || null,
            } : undefined;
          }),
        };
      }
      
      if (sql.includes("SELECT COUNT(*)")) {
        return {
          get: vi.fn(() => ({ count: users.length })),
        };
      }

      if (sql.includes("INSERT INTO users")) {
        return {
          run: vi.fn((...args: any[]) => {
            users.push({
              id: args[0],
              username: args[1],
              password_hash: args[2],
              role: args[3],
              created_at: args[4],
            });
          }),
        };
      }

      if (sql.includes("SELECT password_hash FROM users WHERE id")) {
        return {
          get: vi.fn((id: string) => {
            const user = users.find(u => u.id === id);
            return user ? { password_hash: user.password_hash } : undefined;
          }),
        };
      }

      if (sql.includes("UPDATE users SET password_hash")) {
        return {
          run: vi.fn((hash: string, id: string) => {
            const user = users.find(u => u.id === id);
            if (user) user.password_hash = hash;
          }),
        };
      }

      if (sql.includes("UPDATE users SET last_login")) {
        return {
          run: vi.fn((...args: any[]) => {
            const user = users.find(u => u.id === args[1]);
            if (user) user.last_login = args[0];
          }),
        };
      }

      if (sql.includes("DELETE FROM users")) {
        return {
          run: vi.fn((id: string) => {
            const idx = users.findIndex(u => u.id === id);
            if (idx > -1) users.splice(idx, 1);
          }),
        };
      }

      // Session queries
      if (sql.includes("INSERT INTO sessions")) {
        return {
          run: vi.fn((...args: any[]) => {
            sessions.push({
              id: args[0],
              user_id: args[1],
              token: args[2],
              expires_at: args[3],
              created_at: args[4],
            });
          }),
        };
      }

      if (sql.includes("SELECT s.*, u.username, u.role")) {
        return {
          get: vi.fn((token: string, now: number) => {
            const session = sessions.find(s => s.token === token && s.expires_at > now);
            if (session) {
              const user = users.find(u => u.id === session.user_id);
              return user ? {
                user_id: user.id,
                username: user.username,
                role: user.role,
                created_at: user.created_at,
                last_login: user.last_login || null,
              } : undefined;
            }
            return undefined;
          }),
        };
      }

      if (sql.includes("DELETE FROM sessions WHERE token")) {
        return {
          run: vi.fn((token: string) => {
            const idx = sessions.findIndex(s => s.token === token);
            if (idx > -1) sessions.splice(idx, 1);
          }),
        };
      }

      return { get: vi.fn(() => undefined), run: vi.fn(), all: vi.fn(() => []) };
    }),
    // Expose for testing
    _users: users,
    _sessions: sessions,
  };
}

describe("UserManager", () => {
  let userManager: UserManager;
  let mockDb: any;

  beforeEach(() => {
    mockDb = createMockDb();
    userManager = new UserManager(mockDb);
  });

  describe("createUser", () => {
    it("should create a new user with hashed password", async () => {
      const user = await userManager.createUser({
        username: "testuser",
        password: "password123",
        role: "admin",
      });

      expect(user).toBeDefined();
      expect(user.username).toBe("testuser");
      expect(user.role).toBe("admin");
      expect(user.id).toBeDefined();
      expect(user.createdAt).toBeGreaterThan(0);
    });

    it("should default to admin role if not specified", async () => {
      const user = await userManager.createUser({
        username: "testuser2",
        password: "password123",
      });

      expect(user.role).toBe("admin");
    });

    it("should throw error for duplicate username", async () => {
      await userManager.createUser({
        username: "testuser",
        password: "password123",
      });

      await expect(
        userManager.createUser({
          username: "testuser",
          password: "differentpass",
        })
      ).rejects.toThrow("Username already exists");
    });

    it("should reject short usernames", async () => {
      await expect(
        userManager.createUser({
          username: "ab",
          password: "password123",
        })
      ).rejects.toThrow("at least 3 characters");
    });

    it("should reject weak passwords", async () => {
      await expect(
        userManager.createUser({
          username: "testuser",
          password: "1234567",
        })
      ).rejects.toThrow("at least 8 characters");
    });

    it("should reject empty passwords", async () => {
      await expect(
        userManager.createUser({
          username: "testuser",
          password: "",
        })
      ).rejects.toThrow("at least 8 characters");
    });
  });

  describe("verifyCredentials", () => {
    it("should return user with valid credentials", async () => {
      await userManager.createUser({
        username: "testuser",
        password: "password123",
      });

      const user = await userManager.verifyCredentials("testuser", "password123");

      expect(user).toBeDefined();
      expect(user?.username).toBe("testuser");
    });

    it("should return null for invalid username", async () => {
      const user = await userManager.verifyCredentials("nonexistent", "password123");
      expect(user).toBeNull();
    });

    it("should return null for invalid password", async () => {
      await userManager.createUser({
        username: "testuser",
        password: "password123",
      });

      const user = await userManager.verifyCredentials("testuser", "wrongpassword");
      expect(user).toBeNull();
    });
  });

  describe("updatePassword", () => {
    it("should update password with valid current password", async () => {
      const user = await userManager.createUser({
        username: "testuser",
        password: "oldpassword",
      });

      await userManager.updatePassword(user.id, "oldpassword", "newpassword123");

      // Verify new password works
      const verifiedUser = await userManager.verifyCredentials("testuser", "newpassword123");
      expect(verifiedUser).toBeDefined();
    });

    it("should reject update with invalid current password", async () => {
      const user = await userManager.createUser({
        username: "testuser",
        password: "correctpassword",
      });

      await expect(
        userManager.updatePassword(user.id, "wrongpassword", "newpassword123")
      ).rejects.toThrow("Current password is incorrect");
    });

    it("should enforce password strength on new password", async () => {
      const user = await userManager.createUser({
        username: "testuser",
        password: "oldpassword",
      });

      await expect(
        userManager.updatePassword(user.id, "oldpassword", "1234567")
      ).rejects.toThrow("at least 8 characters");
    });
  });

  describe("hasUsers", () => {
    it("should return false when no users exist", () => {
      expect(userManager.hasUsers()).toBe(false);
    });

    it("should return true when users exist", async () => {
      await userManager.createUser({
        username: "testuser",
        password: "password123",
      });
      expect(userManager.hasUsers()).toBe(true);
    });
  });

  describe("session management", () => {
    it("should create and verify a session", async () => {
      const user = await userManager.createUser({
        username: "testuser",
        password: "password123",
      });

      userManager.createSession(user.id, "test-token", 24);
      const sessionUser = userManager.verifySession("test-token");

      expect(sessionUser).toBeDefined();
      expect(sessionUser?.username).toBe("testuser");
    });

    it("should return null for invalid token", () => {
      const sessionUser = userManager.verifySession("invalid-token");
      expect(sessionUser).toBeNull();
    });

    it("should invalidate session on delete", async () => {
      const user = await userManager.createUser({
        username: "testuser",
        password: "password123",
      });

      userManager.createSession(user.id, "test-token", 24);
      userManager.deleteSession("test-token");

      const sessionUser = userManager.verifySession("test-token");
      expect(sessionUser).toBeNull();
    });
  });

  describe("getUserById", () => {
    it("should return user by ID", async () => {
      const user = await userManager.createUser({
        username: "testuser",
        password: "password123",
      });

      const foundUser = userManager.getUserById(user.id);
      expect(foundUser?.username).toBe("testuser");
    });

    it("should return null for non-existent ID", () => {
      const user = userManager.getUserById("non-existent-id");
      expect(user).toBeNull();
    });
  });
});
