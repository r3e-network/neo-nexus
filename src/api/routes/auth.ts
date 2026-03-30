import { Router, type Request, type Response } from "express";
import type { UserManager } from "../../core/UserManager";
import { createAuthMiddleware, generateToken, type AuthenticatedRequest } from "../middleware/auth";
import { Errors } from '../errors';
import { respondWithApiError } from '../respond';

export function createAuthRouter(userManager: UserManager): Router {
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

      const { username, password } = req.body;

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
      userManager.createSession(user.id, token, 24);

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
      const { username, password } = req.body;

      if (!username || !password) {
        throw Errors.credentialsRequired();
      }

      const user = await userManager.verifyCredentials(username, password);

      if (!user) {
        throw Errors.invalidCredentials();
      }

      // Generate token
      const token = generateToken({
        userId: user.id,
        username: user.username,
      });

      // Create session
      userManager.createSession(user.id, token, 24);
      const usingDefaultPassword = await userManager.isUsingDefaultPassword(user.id);

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
    const authHeader = req.headers.authorization;
    if (authHeader?.startsWith("Bearer ")) {
      const token = authHeader.substring(7);
      userManager.deleteSession(token);
    }
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

      const { username, password, role } = req.body;

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
      userManager.deleteUser(req.params.id as string);
      res.status(204).send();
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}
