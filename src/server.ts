import express from "express";
import cors from "cors";
import rateLimit from "express-rate-limit";
import { createServer as createHttpServer } from "node:http";
import { createServer as createHttpsServer } from "node:https";
import { WebSocketServer, WebSocket } from "ws";
import { loadHttpsConfig, loadHttpsCredentials } from "./config/https";
import type Database from "better-sqlite3";
import { mkdirSync } from "node:fs";
import { paths } from "./utils/paths";
import { NodeManager } from "./core/NodeManager";
import { UserManager } from "./core/UserManager";
import { MetricsCollector } from "./monitoring/MetricsCollector";
import { createNodesRouter } from "./api/routes/nodes";
import { createPluginsRouter } from "./api/routes/plugins";
import { createMetricsRouter } from "./api/routes/metrics";
import { createAuthRouter } from "./api/routes/auth";
import { createPublicRouter } from "./api/routes/public";
import { authMiddleware } from "./api/middleware/auth";

export interface ServerConfig {
  port: number;
  host: string;
  db: Database.Database;
}

export function createAppServer(config: ServerConfig) {
  const app = express();

  // Load HTTPS config and create appropriate server
  const httpsConfig = loadHttpsConfig();
  const httpsCredentials = loadHttpsCredentials(httpsConfig);

  const server = httpsCredentials ? createHttpsServer(httpsCredentials, app) : createHttpServer(app);

  const wss = new WebSocketServer({ server, path: "/ws" });

  // Ensure directories exist
  mkdirSync(paths.base, { recursive: true });
  mkdirSync(paths.nodes, { recursive: true });
  mkdirSync(paths.plugins, { recursive: true });
  mkdirSync(paths.downloads, { recursive: true });
  mkdirSync(paths.logs, { recursive: true });

  // Initialize core services
  const nodeManager = new NodeManager(config.db);
  const userManager = new UserManager(config.db);
  const metricsCollector = new MetricsCollector();

  // Clean up expired sessions periodically
  setInterval(() => {
    userManager.cleanupExpiredSessions();
  }, 60 * 60 * 1000); // Every hour

  // Middleware
  app.use(
    cors({
      origin: process.env.NODE_ENV === "production" ? false : ["http://localhost:3000", "http://127.0.0.1:3000"],
      credentials: true,
    }),
  );

  app.use(express.json({ limit: "10mb" }));

  // Rate limiting
  const limiter = rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 1000,
    message: { error: "Too many requests, please try again later" },
  });
  app.use(limiter);

  // Stricter rate limit for auth endpoints
  const authLimiter = rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 5, // 5 attempts per 15 minutes
    message: { error: "Too many login attempts, please try again later" },
  });

  // Stricter rate limit for node control operations
  const controlLimiter = rateLimit({
    windowMs: 60 * 1000, // 1 minute
    max: 10,
    message: { error: "Too many control operations, please slow down" },
  });

  // Public routes (no auth required) - for view-only access
  app.use("/api/public", createPublicRouter(nodeManager, metricsCollector));

  // Public routes (no auth required)
  app.use("/api/auth", createAuthRouter(userManager));

  // Protected routes middleware
  app.use("/api", (req, res, next) => {
    // Allow login
    if (req.path === "/auth/login") {
      return authLimiter(req, res, next);
    }

    // Require auth for all other routes
    authMiddleware(req, res, next);
  });

  // Protected API Routes
  app.use("/api/nodes", createNodesRouter(nodeManager));
  app.use("/api/nodes/:id/plugins", createPluginsRouter(nodeManager));
  app.use("/api/metrics", createMetricsRouter(nodeManager, metricsCollector));

  // Apply stricter rate limit to control endpoints
  app.use("/api/nodes/:id/start", controlLimiter);
  app.use("/api/nodes/:id/stop", controlLimiter);
  app.use("/api/nodes/:id/restart", controlLimiter);

  // Health check (public)
  app.get("/api/health", (req, res) => {
    res.json({
      status: "ok",
      timestamp: Date.now(),
      nodes: nodeManager.getAllNodes().length,
      authenticated: !!(req as any).user,
    });
  });

  // Get version info (public)
  app.get("/api/version", async (req, res) => {
    try {
      const { DownloadManager } = await import("./core/DownloadManager");
      const [neoCliLatest, neoGoLatest] = await Promise.all([
        DownloadManager.getLatestRelease("neo-cli"),
        DownloadManager.getLatestRelease("neo-go"),
      ]);

      res.json({
        neonexus: "2.0.0",
        latest: {
          "neo-cli": neoCliLatest,
          "neo-go": neoGoLatest,
        },
      });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // Serve static files in production
  if (process.env.NODE_ENV === "production") {
    app.use(express.static("web/dist"));
    app.get("*", (req, res) => {
      res.sendFile("web/dist/index.html", { root: process.cwd() });
    });
  }

  // WebSocket handling
  const clients = new Set<WebSocket>();

  wss.on("connection", (ws) => {
    clients.add(ws);
    console.log("WebSocket client connected");

    ws.on("close", () => {
      clients.delete(ws);
      console.log("WebSocket client disconnected");
    });

    ws.on("error", (error) => {
      console.error("WebSocket error:", error);
      clients.delete(ws);
    });

    // Send initial welcome message
    ws.send(
      JSON.stringify({
        type: "system",
        data: { message: "Connected to NeoNexus Node Manager" },
        timestamp: Date.now(),
      }),
    );
  });

  // Broadcast function
  function broadcast(message: object) {
    const data = JSON.stringify(message);
    clients.forEach((client) => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(data);
      }
    });
  }

  // Periodic metrics broadcast
  setInterval(async () => {
    try {
      const systemMetrics = await metricsCollector.collectSystemMetrics();
      broadcast({
        type: "system",
        data: systemMetrics,
        timestamp: Date.now(),
      });

      // Update and broadcast node metrics
      const nodes = nodeManager.getAllNodes();
      for (const node of nodes) {
        if (node.process.status === "running") {
          await nodeManager.updateMetrics(node.id);
          const updatedNode = nodeManager.getNode(node.id);
          if (updatedNode?.metrics) {
            broadcast({
              type: "metrics",
              nodeId: node.id,
              data: updatedNode.metrics,
              timestamp: Date.now(),
            });
          }
        }
      }
    } catch (error) {
      console.error("Error broadcasting metrics:", error);
    }
  }, 5000); // Every 5 seconds

  // Start server
  function start() {
    return new Promise<void>((resolve) => {
      server.listen(config.port, config.host, () => {
        const protocol = httpsCredentials ? "https" : "http";
        console.log(`🚀 NeoNexus Node Manager running on ${protocol}://${config.host}:${config.port}`);

        // Print security notice for public access
        if (config.host === "0.0.0.0") {
          console.log(`\n⚠️  Security Notice: Server is bound to all interfaces (0.0.0.0)`);
          console.log(`   Access the web interface at: ${protocol}://<your-server-ip>:${config.port}`);
          console.log(`   Make sure port ${config.port} is open in your firewall.\n`);
        }

        // Print default credentials notice
        console.log(`🔐 Default Login Credentials:`);
        console.log(`   Username: admin`);
        console.log(`   Password: admin`);
        console.log(`   ⚠️  IMPORTANT: Please change the default password after first login!\n`);

        resolve();
      });
    });
  }

  // Graceful shutdown
  function stop() {
    return new Promise<void>((resolve) => {
      console.log("Shutting down...");

      // Close all WebSocket connections
      wss.clients.forEach((client) => {
        client.close();
      });

      // Close HTTP server
      server.close(() => {
        console.log("Server closed");
        resolve();
      });

      // Force close after 5 seconds
      setTimeout(() => {
        console.log("Forced shutdown");
        resolve();
      }, 5000);
    });
  }

  return {
    app,
    server,
    wss,
    start,
    stop,
    nodeManager,
    userManager,
    metricsCollector,
  };
}
