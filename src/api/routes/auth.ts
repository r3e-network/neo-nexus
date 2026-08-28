import { Router, type Request, type Response } from "express";
import type { UserManager } from "../../core/UserManager";
import type { AuditEntry, AuditLogger } from "../../core/AuditLogger";
import { isLoopbackAddress } from "../../utils/authSecurity";
import { createAuthMiddleware, generateToken, getTokenExpiresInHours, type AuthenticatedRequest } from "../middleware/auth";
import { Errors } from '../errors';
import { respondWithApiError } from '../respond';

function normalizeUsername(username: unknown): string {
  return typeof username === "string" ? username.trim() : "";
}

function auditAuthentication(
  auditLogger: Pick<AuditLogger, "log"> | undefined,
  req: Request,
  entry: Omit<AuditEntry, "ipAddress">,
): void {
  if (!auditLogger) return;
  try {
    auditLogger.log({ ...entry, ipAddress: req.ip });
  } catch (error) {
    console.error("Authentication audit logging failed:", error instanceof Error ? error.message : error);
  }
}

export function createAuthRouter(
  userManager: UserManager,
  auditLogger?: Pick<AuditLogger, "log">,
): Router {
  const router = Router();
  const requireAuth = createAuthMiddleware(userManager);

  /**
   * POST /api/auth/setup - Initial setup (create first admin user)
   * Only works if no users exist
   */
  router.post("/setup", async (req: Request, res: Response) => {
    try {
      // Check if setup is already complete
      if (userManager.hasUsers()) {
        throw Errors.setupCompleted();
      }

      if (process.env.NODE_ENV === "production" && !isLoopbackAddress(req.ip)) {
        throw Errors.setupLocalOnly();
      }

      const { password } = req.body;
      const username = normalizeUsername(req.body.username);

      if (!username || !password) {
        throw Errors.credentialsRequired();
      }

      const user = await userManager.createUser({
        username,
        password,
        role: "admin",
      });

      // Generate token
      const token = generateToken({
        userId: user.id,
        username: user.username,
      });

      // Create session
      userManager.createSession(user.id, token, getTokenExpiresInHours(token));
      auditAuthentication(auditLogger, req, {
        action: "auth.setup.completed",
        resourceType: "user",
        resourceId: user.id,
        userId: user.id,
        username: user.username,
      });

      res.status(201).json({
        message: "Setup completed successfully",
        user: {
          id: user.id,
          username: user.username,
          role: user.role,
        },
        token,
      });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  /**
   * GET /api/auth/setup-status - Check if initial setup is needed
   */
  router.get("/setup-status", (req: Request, res: Response) => {
    const needsSetup = !userManager.hasUsers();
    res.json({ needsSetup });
  });

  /**
   * POST /api/auth/login - Login user
   */
  router.post("/login", async (req: Request, res: Response) => {
    try {
      const { password } = req.body;
      const username = normalizeUsername(req.body.username);

      if (!username || !password) {
        auditAuthentication(auditLogger, req, {
          action: "auth.login.failed",
          resourceType: "user",
          resourceId: username || undefined,
          username: username || undefined,
          details: JSON.stringify({ reason: "credentials_required" }),
        });
        throw Errors.credentialsRequired();
      }

      const user = await userManager.verifyCredentials(username, password);

      if (!user) {
        auditAuthentication(auditLogger, req, {
          action: "auth.login.failed",
          resourceType: "user",
          resourceId: username,
          username,
          details: JSON.stringify({ reason: "invalid_credentials" }),
        });
        throw Errors.invalidCredentials();
      }

      // Generate token
      const token = generateToken({
        userId: user.id,
        username: user.username,
      });

      // Create session
      userManager.createSession(user.id, token, getTokenExpiresInHours(token));
      const usingDefaultPassword = await userManager.isUsingDefaultPassword(user.id);
      auditAuthentication(auditLogger, req, {
        action: "auth.login.succeeded",
        resourceType: "user",
        resourceId: user.id,
        userId: user.id,
        username: user.username,
        details: JSON.stringify({ usingDefaultPassword }),
      });

      res.json({
        user: {
          id: user.id,
          username: user.username,
          role: user.role,
          usingDefaultPassword,
        },
        token,
      });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  /**
   * POST /api/auth/logout - Logout user
   */
  router.post("/logout", requireAuth, (req: Request, res: Response) => {
    const user = (req as AuthenticatedRequest).user;
    const authHeader = req.headers.authorization;
    if (authHeader?.startsWith("Bearer ")) {
      const token = authHeader.substring(7);
      userManager.deleteSession(token);
    }
    auditAuthentication(auditLogger, req, {
      action: "auth.logout",
      resourceType: "user",
      resourceId: user.id,
      userId: user.id,
      username: user.username,
    });
    res.json({ message: "Logged out successfully" });
  });

  /**
   * POST /api/auth/register - Register new user (admin only)
   */
  router.post("/register", requireAuth, async (req: Request, res: Response) => {
    try {
      // Only admins can register new users
      const user = (req as AuthenticatedRequest).user;
      if (!user || user.role !== "admin") {
        throw Errors.adminRequired();
      }

      const { password, role } = req.body;
      const username = normalizeUsername(req.body.username);

      if (!username || !password) {
        throw Errors.credentialsRequired();
      }

      const validRoles = ["admin", "viewer"];
      const assignedRole = validRoles.includes(role) ? role : "viewer";

      const newUser = await userManager.createUser({
        username,
        password,
        role: assignedRole,
      });
      auditAuthentication(auditLogger, req, {
        action: "auth.user.created",
        resourceType: "user",
        resourceId: newUser.id,
        userId: user.id,
        username: user.username,
        details: JSON.stringify({ createdUsername: newUser.username, role: newUser.role }),
      });

      res.status(201).json({
        user: {
          id: newUser.id,
          username: newUser.username,
          role: newUser.role,
        },
      });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  /**
   * GET /api/auth/me - Get current user
   */
  router.get("/me", requireAuth, async (req: Request, res: Response) => {
    try {
      const user = (req as AuthenticatedRequest).user;
      const usingDefaultPassword = await userManager.isUsingDefaultPassword(user.id);
      res.json({
        user: {
          ...user,
          usingDefaultPassword,
        },
      });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  /**
   * PUT /api/auth/password - Change password
   */
  router.put("/password", requireAuth, async (req: Request, res: Response) => {
    try {
      const user = (req as AuthenticatedRequest).user;
      if (!user) {
        throw Errors.notAuthenticated();
      }

      const { currentPassword, newPassword } = req.body;

      if (!currentPassword || !newPassword) {
        throw Errors.passwordRequired();
      }

      await userManager.updatePassword(user.id, currentPassword, newPassword);
      auditAuthentication(auditLogger, req, {
        action: "auth.password.changed",
        resourceType: "user",
        resourceId: user.id,
        userId: user.id,
        username: user.username,
      });

      res.json({ message: "Password updated successfully" });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  /**
   * GET /api/auth/users - List all users (admin only)
   */
  router.get("/users", requireAuth, (req: Request, res: Response) => {
    try {
      const user = (req as AuthenticatedRequest).user;
      if (!user || user.role !== "admin") {
        throw Errors.adminRequired();
      }

      const users = userManager.getAllUsers();
      res.json({ users });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  /**
   * DELETE /api/auth/users/:id - Delete user (admin only)
   */
  router.delete("/users/:id", requireAuth, (req: Request, res: Response) => {
    try {
      const user = (req as AuthenticatedRequest).user;
      if (!user || user.role !== "admin") {
        throw Errors.adminRequired();
      }

      if (req.params.id === user.id) {
        throw Errors.cannotDeleteSelf();
      }
      const deletedUserId = req.params.id as string;
      userManager.deleteUser(deletedUserId);
      auditAuthentication(auditLogger, req, {
        action: "auth.user.deleted",
        resourceType: "user",
        resourceId: deletedUserId,
        userId: user.id,
        username: user.username,
      });
      res.status(204).send();
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}
