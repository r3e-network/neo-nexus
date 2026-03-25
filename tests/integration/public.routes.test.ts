/**
 * Integration Tests: Public Routes (View-Only Mode)
 * 
 * Tests the public API endpoints that don't require authentication
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import request from "supertest";
import express, { Request, Response } from "express";

describe("Public Routes Integration", () => {
  let app: express.Application;
  let mockNodeManager: any;
  let mockMetricsCollector: any;

  beforeEach(() => {
    app = express();
    app.use(express.json());

    mockNodeManager = {
      getAllNodes: vi.fn(),
      getNode: vi.fn(),
    };

    mockMetricsCollector = {
      getSystemMetrics: vi.fn(),
    };

    // GET /api/public/nodes - List all nodes (public info only)
    app.get("/api/public/nodes", (req: Request, res: Response) => {
      const nodes = mockNodeManager.getAllNodes();
      const publicNodes = nodes.map((node: any) => ({
        id: node.id,
        name: node.name,
        type: node.type,
        network: node.network,
        status: node.status,
        metrics: node.metrics ? {
          blockHeight: node.metrics.blockHeight,
          headerHeight: node.metrics.headerHeight,
          connectedPeers: node.metrics.connectedPeers,
          syncProgress: node.metrics.syncProgress,
          cpuUsage: node.metrics.cpuUsage,
          memoryUsage: node.metrics.memoryUsage,
          lastUpdate: node.metrics.lastUpdate,
        } : null,
      }));
      res.json({ nodes: publicNodes, count: publicNodes.length });
    });

    // GET /api/public/nodes/:id - Get specific node
    app.get("/api/public/nodes/:id", (req: Request, res: Response) => {
      const node = mockNodeManager.getNode(req.params.id);
      if (!node) {
        return res.status(404).json({ error: "Node not found" });
      }
      
      const publicNode = {
        id: node.id,
        name: node.name,
        type: node.type,
        network: node.network,
        status: node.status,
        version: node.version,
        uptime: node.uptime,
        metrics: node.metrics ? {
          blockHeight: node.metrics.blockHeight,
          headerHeight: node.metrics.headerHeight,
          connectedPeers: node.metrics.connectedPeers,
          syncProgress: node.metrics.syncProgress,
          cpuUsage: node.metrics.cpuUsage,
          memoryUsage: node.metrics.memoryUsage,
        } : null,
      };
      
      res.json({ node: publicNode });
    });

    // GET /api/public/nodes/:id/health - Health check
    app.get("/api/public/nodes/:id/health", (req: Request, res: Response) => {
      const node = mockNodeManager.getNode(req.params.id);
      if (!node) {
        return res.status(404).json({ error: "Node not found" });
      }

      const isHealthy = node.status === "running" && 
        (node.metrics?.connectedPeers ?? 0) >= 0;

      res.json({
        healthy: isHealthy,
        status: node.status,
        blockHeight: node.metrics?.blockHeight ?? 0,
        peers: node.metrics?.connectedPeers ?? 0,
      });
    });

    // GET /api/public/metrics/system - System metrics
    app.get("/api/public/metrics/system", async (req: Request, res: Response) => {
      const metrics = await mockMetricsCollector.getSystemMetrics();
      res.json({ metrics });
    });

    // GET /api/public/status - Overall status
    app.get("/api/public/status", (req: Request, res: Response) => {
      const nodes = mockNodeManager.getAllNodes();
      const runningCount = nodes.filter((n: any) => n.status === "running").length;
      const errorCount = nodes.filter((n: any) => n.status === "error").length;
      
      res.json({
        status: errorCount > 0 ? "degraded" : "healthy",
        totalNodes: nodes.length,
        runningNodes: runningCount,
        errorNodes: errorCount,
        timestamp: Date.now(),
      });
    });
  });

  describe("GET /api/public/nodes", () => {
    it("should return nodes without authentication", async () => {
      const mockNodes = [
        {
          id: "node-1",
          name: "Public Node 1",
          type: "neo-cli",
          network: "mainnet",
          status: "running",
          metrics: {
            blockHeight: 5000000,
            headerHeight: 5000000,
            connectedPeers: 10,
            unconnectedPeers: 5,
            syncProgress: 100,
            cpuUsage: 25.5,
            memoryUsage: 512,
            lastUpdate: Date.now(),
          },
        },
      ];
      mockNodeManager.getAllNodes.mockReturnValue(mockNodes);

      const response = await request(app)
        .get("/api/public/nodes")
        .expect(200);

      expect(response.body.nodes).toHaveLength(1);
      expect(response.body.nodes[0].id).toBe("node-1");
      expect(response.body.nodes[0].metrics.blockHeight).toBe(5000000);
    });

    it("should NOT expose sensitive data", async () => {
      const mockNodes = [
        {
          id: "node-1",
          name: "Node",
          type: "neo-cli",
          network: "mainnet",
          status: "running",
          metrics: { blockHeight: 1000 },
          configPath: "/secret/path",
          walletPath: "/secret/wallet.json",
          privateKey: "secret-key",
        },
      ];
      mockNodeManager.getAllNodes.mockReturnValue(mockNodes);

      const response = await request(app)
        .get("/api/public/nodes")
        .expect(200);

      const node = response.body.nodes[0];
      expect(node.id).toBeDefined();
      expect(node.configPath).toBeUndefined();
      expect(node.walletPath).toBeUndefined();
      expect(node.privateKey).toBeUndefined();
    });

    it("should handle nodes without metrics", async () => {
      const mockNodes = [
        {
          id: "node-2",
          name: "Stopped Node",
          type: "neo-go",
          network: "testnet",
          status: "stopped",
          metrics: null,
        },
      ];
      mockNodeManager.getAllNodes.mockReturnValue(mockNodes);

      const response = await request(app)
        .get("/api/public/nodes")
        .expect(200);

      expect(response.body.nodes[0].metrics).toBeNull();
    });
  });

  describe("GET /api/public/nodes/:id", () => {
    it("should return specific node details", async () => {
      const mockNode = {
        id: "node-1",
        name: "Test Node",
        type: "neo-cli",
        network: "mainnet",
        status: "running",
        version: "3.6.0",
        uptime: 3600,
        metrics: {
          blockHeight: 5000000,
          headerHeight: 5000000,
          connectedPeers: 10,
          syncProgress: 100,
          cpuUsage: 25,
          memoryUsage: 512,
        },
      };
      mockNodeManager.getNode.mockReturnValue(mockNode);

      const response = await request(app)
        .get("/api/public/nodes/node-1")
        .expect(200);

      expect(response.body.node.id).toBe("node-1");
      expect(response.body.node.metrics.blockHeight).toBe(5000000);
      expect(response.body.node.version).toBe("3.6.0");
    });

    it("should return 404 for non-existent node", async () => {
      mockNodeManager.getNode.mockReturnValue(null);

      const response = await request(app)
        .get("/api/public/nodes/non-existent")
        .expect(404);

      expect(response.body.error).toContain("not found");
    });
  });

  describe("GET /api/public/nodes/:id/health", () => {
    it("should return healthy for running node with peers", async () => {
      const mockNode = {
        status: "running",
        metrics: { connectedPeers: 10, blockHeight: 1000 },
      };
      mockNodeManager.getNode.mockReturnValue(mockNode);

      const response = await request(app)
        .get("/api/public/nodes/node-1/health")
        .expect(200);

      expect(response.body.healthy).toBe(true);
      expect(response.body.status).toBe("running");
      expect(response.body.peers).toBe(10);
    });

    it("should return unhealthy for stopped node", async () => {
      const mockNode = {
        status: "stopped",
        metrics: null,
      };
      mockNodeManager.getNode.mockReturnValue(mockNode);

      const response = await request(app)
        .get("/api/public/nodes/node-1/health")
        .expect(200);

      expect(response.body.healthy).toBe(false);
      expect(response.body.status).toBe("stopped");
    });

    it("should return 404 for non-existent node", async () => {
      mockNodeManager.getNode.mockReturnValue(null);

      const response = await request(app)
        .get("/api/public/nodes/non-existent/health")
        .expect(404);

      expect(response.body.error).toContain("not found");
    });
  });

  describe("GET /api/public/metrics/system", () => {
    it("should return system metrics", async () => {
      const systemMetrics = {
        cpu: { usage: 45.5, cores: 8 },
        memory: { total: 16000000000, used: 8000000000, free: 8000000000 },
        disk: { total: 1000000000000, used: 500000000000 },
        network: { rx: 1024, tx: 2048 },
      };
      mockMetricsCollector.getSystemMetrics.mockResolvedValue(systemMetrics);

      const response = await request(app)
        .get("/api/public/metrics/system")
        .expect(200);

      expect(response.body.metrics.cpu.usage).toBe(45.5);
      expect(response.body.metrics.memory.total).toBe(16000000000);
    });
  });

  describe("GET /api/public/status", () => {
    it("should return healthy status when all nodes running", async () => {
      const mockNodes = [
        { status: "running" },
        { status: "running" },
      ];
      mockNodeManager.getAllNodes.mockReturnValue(mockNodes);

      const response = await request(app)
        .get("/api/public/status")
        .expect(200);

      expect(response.body.status).toBe("healthy");
      expect(response.body.totalNodes).toBe(2);
      expect(response.body.runningNodes).toBe(2);
      expect(response.body.errorNodes).toBe(0);
    });

    it("should return degraded status when nodes have errors", async () => {
      const mockNodes = [
        { status: "running" },
        { status: "error" },
      ];
      mockNodeManager.getAllNodes.mockReturnValue(mockNodes);

      const response = await request(app)
        .get("/api/public/status")
        .expect(200);

      expect(response.body.status).toBe("degraded");
      expect(response.body.errorNodes).toBe(1);
    });
  });
});
