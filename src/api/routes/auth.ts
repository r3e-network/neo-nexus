import { Router, type Request, type Response } from "express";
import type { UserManager } from "../../core/UserManager";
import { generateToken } from "../middleware/auth";

export function createAuthRouter(userManager: UserManager): Router {
  const router = Router();

  /**
   * POST /api/auth/setup - Initial setup (create first admin user)
   * Only works if no users exist
   */
  router.post("/setup", async (req: Request, res: Response) => {
    try {
      // Check if setup is already complete
      if (userManager.hasUsers()) {
        return res.status(403).json({
          error: "Setup already completed. Use /register to create new users.",
        });
      }

      const { username, password } = req.body;

      if (!username || !password) {
        return res.status(400).json({ error: "Username and password are required" });
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
      res.status(400).json({ error: String(error) });
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
        return res.status(400).json({ error: "Username and password are required" });
      }

      const user = await userManager.verifyCredentials(username, password);

      if (!user) {
        return res.status(401).json({ error: "Invalid credentials" });
      }

      // Generate token
      const token = generateToken({
        userId: user.id,
        username: user.username,
      });

      // Create session
      userManager.createSession(user.id, token, 24);

      res.json({
        user: {
          id: user.id,
          username: user.username,
          role: user.role,
        },
        token,
      });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  /**
   * POST /api/auth/logout - Logout user
   */
  router.post("/logout", (req: Request, res: Response) => {
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
  router.post("/register", async (req: Request, res: Response) => {
    try {
      // Only admins can register new users
      const user = (req as any).user;
      if (!user || user.role !== "admin") {
        return res.status(403).json({ error: "Admin access required" });
      }

      const { username, password, role } = req.body;

      if (!username || !password) {
        return res.status(400).json({ error: "Username and password are required" });
      }

      const newUser = await userManager.createUser({
        username,
        password,
        role: role || "viewer",
      });

      res.status(201).json({
        user: {
          id: newUser.id,
          username: newUser.username,
          role: newUser.role,
        },
      });
    } catch (error) {
      res.status(400).json({ error: String(error) });
    }
  });

  /**
   * GET /api/auth/me - Get current user
   */
  router.get("/me", (req: Request, res: Response) => {
    const user = (req as any).user;
    if (!user) {
      return res.status(401).json({ error: "Not authenticated" });
    }
    res.json({ user });
  });

  /**
   * PUT /api/auth/password - Change password
   */
  router.put("/password", async (req: Request, res: Response) => {
    try {
      const user = (req as any).user;
      if (!user) {
        return res.status(401).json({ error: "Not authenticated" });
      }

      const { currentPassword, newPassword } = req.body;

      if (!currentPassword || !newPassword) {
        return res.status(400).json({ error: "Current and new password are required" });
      }

      await userManager.updatePassword(user.userId, currentPassword, newPassword);

      res.json({ message: "Password updated successfully" });
    } catch (error) {
      res.status(400).json({ error: String(error) });
    }
  });

  /**
   * GET /api/auth/users - List all users (admin only)
   */
  router.get("/users", (req: Request, res: Response) => {
    const user = (req as any).user;
    if (!user || user.role !== "admin") {
      return res.status(403).json({ error: "Admin access required" });
    }

    const users = userManager.getAllUsers();
    res.json({ users });
  });

  /**
   * DELETE /api/auth/users/:id - Delete user (admin only)
   */
  router.delete("/users/:id", (req: Request, res: Response) => {
    const user = (req as any).user;
    if (!user || user.role !== "admin") {
      return res.status(403).json({ error: "Admin access required" });
    }

    try {
      userManager.deleteUser(req.params.id as string);
      res.status(204).send();
    } catch (error) {
      res.status(400).json({ error: String(error) });
    }
  });

  return router;
}
