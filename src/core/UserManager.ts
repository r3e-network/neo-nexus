import type Database from "better-sqlite3";
import bcrypt from "bcrypt";
import { randomUUID } from "node:crypto";

export interface User {
  id: string;
  username: string;
  role: "admin" | "viewer";
  createdAt: number;
  lastLogin?: number;
  usingDefaultPassword?: boolean;
}

export interface CreateUserRequest {
  username: string;
  password: string;
  role?: "admin" | "viewer";
}

export class UserManager {
  constructor(private db: Database.Database) {}

  /**
   * Create a new user
   */
  async createUser(request: CreateUserRequest): Promise<User> {
    // Validate username
    if (!request.username || request.username.length < 3) {
      throw new Error("Username must be at least 3 characters");
    }

    // Validate password
    if (!request.password || request.password.length < 8) {
      throw new Error("Password must be at least 8 characters");
    }

    // Check if username already exists
    const existing = this.db.prepare("SELECT id FROM users WHERE username = ?").get(request.username);
    if (existing) {
      throw new Error("Username already exists");
    }

    // Hash password
    const passwordHash = await bcrypt.hash(request.password, 10);

    const userId = randomUUID();
    const now = Date.now();

    const stmt = this.db.prepare(`
      INSERT INTO users (id, username, password_hash, role, created_at)
      VALUES (?, ?, ?, ?, ?)
    `);

    stmt.run(userId, request.username, passwordHash, request.role || "admin", now);

    return {
      id: userId,
      username: request.username,
      role: request.role || "admin",
      createdAt: now,
    };
  }

  /**
   * Verify user credentials
   */
  async verifyCredentials(username: string, password: string): Promise<User | null> {
    const row = this.db.prepare("SELECT * FROM users WHERE username = ?").get(username) as
      | {
          id: string;
          username: string;
          password_hash: string;
          role: string;
          created_at: number;
          last_login: number | null;
        }
      | undefined;

    if (!row) {
      return null;
    }

    const valid = await bcrypt.compare(password, row.password_hash);
    if (!valid) {
      return null;
    }

    // Update last login
    this.db.prepare("UPDATE users SET last_login = ? WHERE id = ?").run(Date.now(), row.id);

    return {
      id: row.id,
      username: row.username,
      role: row.role as "admin" | "viewer",
      createdAt: row.created_at,
      lastLogin: row.last_login || undefined,
    };
  }

  /**
   * Get user by ID
   */
  getUserById(userId: string): User | null {
    const row = this.db.prepare("SELECT * FROM users WHERE id = ?").get(userId) as
      | {
          id: string;
          username: string;
          role: string;
          created_at: number;
          last_login: number | null;
        }
      | undefined;

    if (!row) return null;

    return {
      id: row.id,
      username: row.username,
      role: row.role as "admin" | "viewer",
      createdAt: row.created_at,
      lastLogin: row.last_login || undefined,
    };
  }

  /**
   * Get all users
   */
  getAllUsers(): User[] {
    const rows = this.db.prepare("SELECT * FROM users ORDER BY created_at").all() as Array<{
      id: string;
      username: string;
      role: string;
      created_at: number;
      last_login: number | null;
    }>;

    return rows.map((row) => ({
      id: row.id,
      username: row.username,
      role: row.role as "admin" | "viewer",
      createdAt: row.created_at,
      lastLogin: row.last_login || undefined,
    }));
  }

  /**
   * Update user password
   */
  async updatePassword(userId: string, currentPassword: string, newPassword: string): Promise<void> {
    if (!newPassword || newPassword.length < 8) {
      throw new Error("New password must be at least 8 characters");
    }

    // Verify current password
    const row = this.db.prepare("SELECT password_hash FROM users WHERE id = ?").get(userId) as
      | { password_hash: string }
      | undefined;

    if (!row) {
      throw new Error("User not found");
    }

    const valid = await bcrypt.compare(currentPassword, row.password_hash);
    if (!valid) {
      throw new Error("Current password is incorrect");
    }

    // Hash new password
    const newHash = await bcrypt.hash(newPassword, 10);
    this.db.prepare("UPDATE users SET password_hash = ? WHERE id = ?").run(newHash, userId);
  }

  /**
   * Delete user
   */
  deleteUser(userId: string): void {
    // Prevent deleting the last admin
    const adminCount = this.db.prepare("SELECT COUNT(*) as count FROM users WHERE role = 'admin'").get() as {
      count: number;
    };
    const user = this.getUserById(userId);

    if (user?.role === "admin" && adminCount.count <= 1) {
      throw new Error("Cannot delete the last admin user");
    }

    this.db.prepare("DELETE FROM users WHERE id = ?").run(userId);
  }

  /**
   * Check if any users exist
   */
  hasUsers(): boolean {
    const result = this.db.prepare("SELECT COUNT(*) as count FROM users").get() as { count: number };
    return result.count > 0;
  }

  /**
   * Create session
   */
  createSession(userId: string, token: string, expiresInHours: number = 24): void {
    const sessionId = randomUUID();
    const now = Date.now();
    const expiresAt = now + expiresInHours * 60 * 60 * 1000;

    this.db.prepare(`
      INSERT INTO sessions (id, user_id, token, expires_at, created_at)
      VALUES (?, ?, ?, ?, ?)
    `).run(sessionId, userId, token, expiresAt, now);
  }

  /**
   * Verify session token
   */
  verifySession(token: string): User | null {
    const row = this.db.prepare(`
      SELECT s.*, u.username, u.role, u.created_at, u.last_login
      FROM sessions s
      JOIN users u ON s.user_id = u.id
      WHERE s.token = ? AND s.expires_at > ?
    `).get(token, Date.now()) as
      | {
          user_id: string;
          username: string;
          role: string;
          created_at: number;
          last_login: number | null;
        }
      | undefined;

    if (!row) {
      return null;
    }

    return {
      id: row.user_id,
      username: row.username,
      role: row.role as "admin" | "viewer",
      createdAt: row.created_at,
      lastLogin: row.last_login || undefined,
    };
  }

  /**
   * Delete session
   */
  deleteSession(token: string): void {
    this.db.prepare("DELETE FROM sessions WHERE token = ?").run(token);
  }

  /**
   * Clean up expired sessions
   */
  cleanupExpiredSessions(): void {
    this.db.prepare("DELETE FROM sessions WHERE expires_at < ?").run(Date.now());
  }

  /**
   * Check whether a user still uses the default admin password
   */
  async isUsingDefaultPassword(userId: string): Promise<boolean> {
    const row = this.db.prepare("SELECT password_hash FROM users WHERE id = ?").get(userId) as
      | { password_hash: string }
      | undefined;

    if (!row) {
      return false;
    }

    return bcrypt.compare("admin", row.password_hash);
  }
}
