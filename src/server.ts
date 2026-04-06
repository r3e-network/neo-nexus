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
import { writePidFile, removePidFile } from "./utils/lifecycle";
import { NodeManager } from "./core/NodeManager";
import { UserManager } from "./core/UserManager";
import { MetricsCollector } from "./monitoring/MetricsCollector";
import { createNodesRouter } from "./api/routes/nodes";
import { createPluginsRouter } from "./api/routes/plugins";
import { createMetricsRouter } from "./api/routes/metrics";
import { createAuthRouter } from "./api/routes/auth";
import { createPublicRouter } from "./api/routes/public";
import { createAuthMiddleware, verifyToken, type AuthenticatedRequest } from "./api/middleware/auth";
import { createSystemRouter } from "./api/routes/system";
import { createServersRouter } from "./api/routes/servers";
import { createSecureSignersRouter } from "./api/routes/secureSigners";
import { buildNodeLogMessage, buildNodeMetricsMessage, buildNodeStatusMessage, buildSystemMessage } from "./realtime/messages";
import { RemoteServerManager } from "./core/RemoteServerManager";
import { NetworkHeightTracker } from "./core/NetworkHeightTracker";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { pruneAllLogs } from "./utils/logRetention";
import { recordDiskReading, getDiskAlertLevel } from "./utils/diskMonitor";
import { AuditLogger } from "./core/AuditLogger";
import { WatchdogManager } from "./core/WatchdogManager";
import { IntegrationManager } from "./integrations/IntegrationManager";
import { createIntegrationsRouter } from "./api/routes/integrations";
import type { IntegrationEvent, LogEntryWithContext } from "./integrations/types";
import { printShutdownMessage } from "./utils/startup";

const pkg = JSON.parse(readFileSync(join(import.meta.dirname ?? ".", "..", "package.json"), "utf-8"));
const APP_VERSION: string = pkg.version || "0.0.0";

/** Rate limiting configuration */
const RATE_LIMIT_WINDOW_MS = 15 * 60 * 1000;
const RATE_LIMIT_MAX_REQUESTS = 1000;
const AUTH_RATE_LIMIT_MAX = 5;
const CONTROL_RATE_LIMIT_WINDOW_MS = 60 * 1000;
const CONTROL_RATE_LIMIT_MAX = 10;

/** Periodic task intervals */
const SESSION_CLEANUP_INTERVAL_MS = 60 * 60 * 1000;
const METRICS_BROADCAST_INTERVAL_MS = 5_000;
const LOG_RETENTION_INTERVAL_MS = 10 * 60 * 1000;
const NETWORK_HEIGHT_INTERVAL_MS = 60 * 1000;

/** Shutdown configuration */
const GRACEFUL_SHUTDOWN_TIMEOUT_MS = 5_000;
const FORCE_EXIT_TIMEOUT_MS = 30_000;

/** Log retention defaults */
const DEFAULT_LOG_RETENTION_MAX_ROWS = 50_000;
const DEFAULT_AUDIT_LOG_MAX_ROWS = 100_000;
const DEFAULT_LOG_PRUNE_BATCH_SIZE = 500_000;

export interface ServerConfig {
  port: number;
  host: string;
  db: Database.Database;
}

export function createAppServer(config: ServerConfig) {
  const app = express();
  app.set('appVersion', APP_VERSION);

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
  nodeManager.reconcileProcessStates();

  const watchdog = new WatchdogManager(async (nodeId) => {
    try {
      await nodeManager.startNode(nodeId);
    } catch (error) {
      console.error(`Watchdog restart failed for ${nodeId}:`, error);
    }
  });
  const secureSignerManager = nodeManager.getSecureSignerManager();
  const remoteServerManager = new RemoteServerManager(config.db);
  const userManager = new UserManager(config.db);
  const metricsCollector = new MetricsCollector();
  const networkHeightTracker = new NetworkHeightTracker();
  const auditLogger = new AuditLogger(config.db);
  const integrationManager = new IntegrationManager(config.db);
  const logBuffer: LogEntryWithContext[] = [];
  const requireAuth = createAuthMiddleware(userManager);

  // Clean up expired sessions periodically
  const sessionCleanupInterval = setInterval(() => {
    userManager.cleanupExpiredSessions();
  }, SESSION_CLEANUP_INTERVAL_MS); // Every hour

  // Middleware
  const corsOrigin = process.env.CORS_ORIGIN
    ? process.env.CORS_ORIGIN.split(",").map((s) => s.trim())
    : process.env.NODE_ENV === "production"
      ? false
      : ["http://localhost:3000", "http://127.0.0.1:3000"];

  app.use(
    cors({
      origin: corsOrigin,
      credentials: true,
    }),
  );

  app.use(express.json({ limit: "10mb" }));

  // Rate limiting
  const limiter = rateLimit({
    windowMs: RATE_LIMIT_WINDOW_MS,
    max: RATE_LIMIT_MAX_REQUESTS,
    message: { error: "Too many requests, please try again later" },
  });
  app.use(limiter);

  // Stricter rate limit for auth endpoints
  const authLimiter = rateLimit({
    windowMs: RATE_LIMIT_WINDOW_MS,
    max: AUTH_RATE_LIMIT_MAX, // 5 attempts per 15 minutes
    message: { error: "Too many login attempts, please try again later" },
  });
  app.use("/api/auth/login", authLimiter);

  // Stricter rate limit for node control operations
  const controlLimiter = rateLimit({
    windowMs: CONTROL_RATE_LIMIT_WINDOW_MS,
    max: CONTROL_RATE_LIMIT_MAX,
    message: { error: "Too many control operations, please slow down" },
  });

  // Public routes (no auth required) - for view-only access
  app.use("/api/public", createPublicRouter(nodeManager, metricsCollector));

  // Public routes (no auth required)
  app.use("/api/auth", createAuthRouter(userManager));

  // Apply stricter rate limit to control endpoints
  app.use("/api/nodes/:id/start", requireAuth, controlLimiter);
  app.use("/api/nodes/:id/stop", requireAuth, controlLimiter);
  app.use("/api/nodes/:id/restart", requireAuth, controlLimiter);

  // Health check (public, with optional auth detection)
  app.get("/api/health", (req, res) => {
    let authenticated = false;
    try {
      const authHeader = req.headers.authorization;
      if (authHeader?.startsWith("Bearer ")) {
        const token = authHeader.substring(7);
        const session = userManager.verifySession(token);
        authenticated = !!session;
      }
    } catch {
      // Token invalid — still report health, just not authenticated
    }
    res.json({
      status: "ok",
      timestamp: Date.now(),
      nodes: nodeManager.getAllNodes().length,
      authenticated,
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
        neonexus: APP_VERSION,
        latest: {
          "neo-cli": neoCliLatest,
          "neo-go": neoGoLatest,
        },
      });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  // Admin-only middleware
  function requireAdmin(req: express.Request, res: express.Response, next: express.NextFunction) {
    const user = (req as AuthenticatedRequest).user;
    if (!user || user.role !== "admin") {
      return res.status(403).json({ error: "Admin access required" });
    }
    next();
  }

  // Protected API Routes
  app.use("/api/nodes", requireAuth, createNodesRouter(nodeManager));
  app.use("/api/nodes/:id/plugins", requireAuth, createPluginsRouter(nodeManager));
  app.use("/api/metrics", requireAuth, createMetricsRouter(nodeManager, metricsCollector));
  app.use("/api/system", requireAuth, requireAdmin, createSystemRouter(nodeManager));
  app.use("/api/servers", requireAuth, createServersRouter(remoteServerManager));
  app.use("/api/secure-signers", requireAuth, requireAdmin, createSecureSignersRouter(secureSignerManager));
  app.use("/api/integrations", requireAuth, requireAdmin, createIntegrationsRouter(integrationManager));

  // Audit log endpoint
  app.get("/api/system/audit-log", requireAuth, (req, res) => {
    const limit = Math.min(parseInt(String(req.query.limit ?? "100"), 10), 1000);
    const offset = parseInt(String(req.query.offset ?? "0"), 10);
    const entries = auditLogger.query({ limit, offset });
    res.json({ entries });
  });

  // Network height endpoint
  app.get("/api/metrics/network", requireAuth, (_req, res) => {
    res.json({
      mainnet: networkHeightTracker.getHeight("mainnet"),
      testnet: networkHeightTracker.getHeight("testnet"),
      timestamp: Date.now(),
    });
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

  wss.on("connection", (ws, req) => {
    // Authenticate WebSocket connections via query parameter token
    try {
      const url = new URL(req.url || "", `http://${req.headers.host}`);
      const token = url.searchParams.get("token");
      if (!token) {
        ws.close(4001, "Authentication required");
        return;
      }
      verifyToken(token);
      const sessionUser = userManager.verifySession(token);
      if (!sessionUser) {
        ws.close(4001, "Invalid or expired session");
        return;
      }
    } catch {
      ws.close(4001, "Invalid token");
      return;
    }

    clients.add(ws);

    ws.on("close", () => {
      clients.delete(ws);
    });

    ws.on("error", (error) => {
      console.error("WebSocket error:", error);
      clients.delete(ws);
    });

    ws.send(JSON.stringify({ type: "welcome", message: "Connected to NeoNexus Node Manager", timestamp: Date.now() }));
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

  nodeManager.on("nodeStatus", ({ nodeId, status, previousStatus }) => {
    broadcast(buildNodeStatusMessage(nodeId, status, previousStatus));
    if (status === "running" && previousStatus === "starting") {
      auditLogger.log({ action: "node.start", resourceType: "node", resourceId: nodeId });
    } else if (status === "stopped" && previousStatus === "stopping") {
      auditLogger.log({ action: "node.stop", resourceType: "node", resourceId: nodeId });
    }

    if (status === "running") watchdog.onNodeStarted(nodeId);
    if (status === "error" || status === "stopped") {
      const wasExpected = previousStatus === "stopping";
      watchdog.onNodeExited(nodeId, wasExpected);
    }

    // Integration notifications — single notification per event
    const statusNode = nodeManager.getNode(nodeId);
    const statusNodeName = statusNode?.name ?? nodeId;
    let integrationEvent: IntegrationEvent | null = null;

    if (status === "running" && previousStatus === "starting") {
      integrationEvent = { type: 'node.started', severity: 'info', title: 'Node Started', message: `Node ${statusNodeName} has started.`, nodeId, nodeName: statusNodeName, timestamp: Date.now() };
    } else if (status === "stopped" && previousStatus === "stopping") {
      integrationEvent = { type: 'node.stopped', severity: 'info', title: 'Node Stopped', message: `Node ${statusNodeName} has stopped.`, nodeId, nodeName: statusNodeName, timestamp: Date.now() };
    } else if (status === "error" || (status === "stopped" && previousStatus !== "stopping")) {
      // Unexpected exit — use watchdog state to pick the right event type
      if (watchdog.isExhausted(nodeId)) {
        integrationEvent = { type: 'watchdog.exhausted', severity: 'critical', title: 'Watchdog Exhausted', message: `Node ${statusNodeName} crashed too many times. Watchdog giving up.`, nodeId, nodeName: statusNodeName, timestamp: Date.now() };
      } else {
        integrationEvent = { type: 'node.crashed', severity: 'critical', title: 'Node Crashed', message: `Node ${statusNodeName} crashed unexpectedly. Watchdog scheduling restart.`, nodeId, nodeName: statusNodeName, timestamp: Date.now() };
      }
    }

    if (integrationEvent) {
      integrationManager.broadcastNotification(integrationEvent).catch(() => {});
    }
  });

  nodeManager.on("nodeLog", ({ nodeId, entry }) => {
    broadcast(buildNodeLogMessage(nodeId, entry));
    const logNode = nodeManager.getNode(nodeId);
    logBuffer.push({ ...entry, nodeId, nodeName: logNode?.name ?? nodeId });
  });

  nodeManager.on("nodeMetrics", ({ nodeId, metrics }) => {
    broadcast(buildNodeMetricsMessage(nodeId, metrics));
  });

  // Periodic metrics broadcast
  const metricsInterval = setInterval(async () => {
    try {
      const systemMetrics = await metricsCollector.collectSystemMetrics();
      broadcast(buildSystemMessage(systemMetrics));

      // Flush buffered logs to integration providers
      if (logBuffer.length > 0) {
        const batch = logBuffer.splice(0, logBuffer.length);
        integrationManager.broadcastLogs(batch).catch(() => {});
      }

      // Push system + node metrics to integration providers
      const allNodes = nodeManager.getAllNodes();
      const nodeMetricsForIntegration = allNodes
        .filter(n => n.process.status === "running" && n.metrics)
        .map(n => ({
          nodeId: n.id,
          nodeName: n.name,
          nodeType: n.type,
          network: n.network,
          metrics: n.metrics!,
        }));
      integrationManager.broadcastMetrics(systemMetrics, nodeMetricsForIntegration).catch(() => {});

      // Record disk reading and log alerts if needed
      recordDiskReading(systemMetrics.disk.free);
      const diskAlert = getDiskAlertLevel(systemMetrics.disk.percentage);
      if (diskAlert !== null) {
        console.warn(`[disk] ${diskAlert.toUpperCase()}: disk usage at ${systemMetrics.disk.percentage.toFixed(1)}%`);

        integrationManager.broadcastNotification({
          type: diskAlert === 'critical' ? 'disk.critical' : 'disk.warning',
          severity: diskAlert === 'critical' ? 'critical' : 'warning',
          title: `Disk ${diskAlert === 'critical' ? 'Critical' : 'Warning'}`,
          message: `Disk usage at ${systemMetrics.disk.percentage.toFixed(1)}%`,
          timestamp: Date.now(),
        }).catch(() => {});
      }

      // Update and broadcast node metrics
      const nodes = nodeManager.getAllNodes();
      for (const node of nodes) {
        if (node.process.status === "running") {
          await nodeManager.updateMetrics(node.id);
          // Compute and persist sync progress
          const updated = nodeManager.getNode(node.id);
          const blockHeight = updated?.metrics?.blockHeight ?? 0;
          if (blockHeight > 0 && node.network !== "private") {
            networkHeightTracker.recordNodeHeight(node.id, blockHeight);
            const syncProgress = networkHeightTracker.getSyncProgress(blockHeight, node.network);
            nodeManager.updateSyncProgress(node.id, syncProgress);
          }
        }
      }
    } catch (error) {
      console.error("Error broadcasting metrics:", error);
    }
  }, METRICS_BROADCAST_INTERVAL_MS); // Every 5 seconds

  // Periodically prune logs to stay within configured retention limits
  const maxPerNode = parseInt(process.env.LOG_RETENTION_MAX_ROWS || String(DEFAULT_LOG_RETENTION_MAX_ROWS), 10);
  const logRetentionInterval = setInterval(() => {
    try {
      pruneAllLogs(config.db, maxPerNode, DEFAULT_LOG_PRUNE_BATCH_SIZE);
      auditLogger.prune(DEFAULT_AUDIT_LOG_MAX_ROWS);
    } catch (error) {
      console.error("Error pruning logs:", error);
    }
  }, LOG_RETENTION_INTERVAL_MS); // Every 10 minutes

  // Periodically fetch network heights from seed nodes
  const networkHeightInterval = setInterval(async () => {
    try {
      const [mainnetHeight, testnetHeight] = await Promise.allSettled([
        networkHeightTracker.fetchNetworkHeight("mainnet"),
        networkHeightTracker.fetchNetworkHeight("testnet"),
      ]);
      if (mainnetHeight.status === "fulfilled" && mainnetHeight.value !== null) {
        networkHeightTracker.setHeight("mainnet", mainnetHeight.value);
      }
      if (testnetHeight.status === "fulfilled" && testnetHeight.value !== null) {
        networkHeightTracker.setHeight("testnet", testnetHeight.value);
      }
    } catch (error) {
      console.error("Error fetching network heights:", error);
    }
  }, NETWORK_HEIGHT_INTERVAL_MS); // Every 60 seconds

  // Start server
  function start() {
    return new Promise<void>((resolve) => {
      server.listen(config.port, config.host, () => {
        resolve();
      });
    });
  }

  // Graceful shutdown
  async function stop() {
    // Clear intervals
    clearInterval(sessionCleanupInterval);
    clearInterval(metricsInterval);
    clearInterval(networkHeightInterval);
    clearInterval(logRetentionInterval);
    watchdog.clearAll();
    integrationManager.shutdown();

    // Stop all running nodes
    let stoppedCount = 0;
    try {
      const result = await nodeManager.stopAllNodes();
      stoppedCount = result.stoppedCount;
    } catch (error) {
      console.error("Error stopping nodes:", error);
    }

    printShutdownMessage(stoppedCount);

    // Close all WebSocket connections
    wss.clients.forEach((client) => {
      client.close();
    });

    return new Promise<void>((resolve) => {
      // Close HTTP server
      server.close(() => {
        // Close database
        try {
          config.db.close();
        } catch {
          // ignore
        }
        resolve();
      });

      // Force close after timeout
      setTimeout(() => {
        resolve();
      }, GRACEFUL_SHUTDOWN_TIMEOUT_MS);
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
    integrationManager,
  };
}

/**
 * High-level entry point: creates the app server, starts it, writes a PID
 * file, and registers SIGTERM/SIGINT handlers for graceful shutdown.
 *
 * Called from src/index.ts.
 */
export async function startServer(config: ServerConfig): Promise<ReturnType<typeof createAppServer>> {
  const appServer = createAppServer(config);

  await appServer.start();

  // Write PID file so external tooling can find the process
  const pidFile = join(paths.base, "neonexus.pid");
  writePidFile(pidFile);

  let shuttingDown = false;

  async function shutdown(_signal: string): Promise<void> {
    if (shuttingDown) return;
    shuttingDown = true;

    // Force-exit safety net — 30 seconds
    const forceExit = setTimeout(() => {
      console.error("Graceful shutdown timed out after 30 s — forcing exit");
      process.exit(1);
    }, FORCE_EXIT_TIMEOUT_MS);
    // Allow the event loop to exit even if this timer is still pending
    forceExit.unref();

    try {
      await appServer.stop();
    } finally {
      removePidFile(pidFile);
      process.exit(0);
    }
  }

  process.on("SIGTERM", () => shutdown("SIGTERM"));
  process.on("SIGINT", () => shutdown("SIGINT"));

  return appServer;
}
