import { Router, type Request, type Response } from "express";
import type { NodeManager } from "../../core/NodeManager";
import type { MetricsCollector } from "../../monitoring/MetricsCollector";

export function createPublicRouter(nodeManager: NodeManager, metricsCollector: MetricsCollector): Router {
  const router = Router();

  /**
   * GET /api/public/status - Overall system status
   * Returns summary of all nodes without sensitive details
   */
  router.get("/status", (req: Request, res: Response) => {
    try {
      const nodes = nodeManager.getAllNodes();
      
      const summary = {
        totalNodes: nodes.length,
        runningNodes: nodes.filter(n => n.process.status === "running").length,
        syncingNodes: nodes.filter(n => n.process.status === "syncing").length,
        errorNodes: nodes.filter(n => n.process.status === "error").length,
        totalBlocks: nodes.reduce((sum, n) => sum + (n.metrics?.blockHeight || 0), 0),
        totalPeers: nodes.reduce((sum, n) => sum + (n.metrics?.connectedPeers || 0), 0),
        timestamp: Date.now(),
      };

      res.json({ status: summary });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  /**
   * GET /api/public/nodes - List all nodes (public info only)
   */
  router.get("/nodes", (req: Request, res: Response) => {
    try {
      const nodes = nodeManager.getAllNodes();
      
      // Return only public-safe information
      const publicNodes = nodes.map(node => ({
        id: node.id,
        name: node.name,
        type: node.type,
        network: node.network,
        status: node.process.status,
        version: node.version,
        metrics: node.metrics ? {
          blockHeight: node.metrics.blockHeight,
          headerHeight: node.metrics.headerHeight,
          connectedPeers: node.metrics.connectedPeers,
          syncProgress: node.metrics.syncProgress,
        } : null,
        uptime: node.process.uptime,
        lastUpdate: node.metrics?.lastUpdate,
      }));

      res.json({ nodes: publicNodes });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  /**
   * GET /api/public/nodes/:id - Get specific node details (public)
   */
  router.get("/nodes/:id", (req: Request, res: Response) => {
    try {
      const node = nodeManager.getNode(req.params.id as string);
      if (!node) {
        return res.status(404).json({ error: "Node not found" });
      }

      // Return only public-safe information
      const publicNode = {
        id: node.id,
        name: node.name,
        type: node.type,
        network: node.network,
        syncMode: node.syncMode,
        status: node.process.status,
        version: node.version,
        metrics: node.metrics ? {
          blockHeight: node.metrics.blockHeight,
          headerHeight: node.metrics.headerHeight,
          connectedPeers: node.metrics.connectedPeers,
          unconnectedPeers: node.metrics.unconnectedPeers,
          syncProgress: node.metrics.syncProgress,
          cpuUsage: node.metrics.cpuUsage,
          memoryUsage: node.metrics.memoryUsage,
          lastUpdate: node.metrics.lastUpdate,
        } : null,
        uptime: node.process.uptime,
        createdAt: node.createdAt,
      };

      res.json({ node: publicNode });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  /**
   * GET /api/public/nodes/:id/health - Node health check (public)
   */
  router.get("/nodes/:id/health", (req: Request, res: Response) => {
    try {
      const node = nodeManager.getNode(req.params.id as string);
      if (!node) {
        return res.status(404).json({ error: "Node not found" });
      }

      const isHealthy = node.process.status === "running" &&
        (node.metrics?.connectedPeers ?? 0) > 0;

      res.json({
        healthy: isHealthy,
        status: node.process.status,
        blockHeight: node.metrics?.blockHeight ?? 0,
        peers: node.metrics?.connectedPeers ?? 0,
        timestamp: Date.now(),
      });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  /**
   * GET /api/public/metrics/system - System metrics (public)
   */
  router.get("/metrics/system", async (req: Request, res: Response) => {
    try {
      const metrics = await metricsCollector.collectSystemMetrics();
      
      // Return simplified system metrics
      const publicMetrics = {
        cpu: {
          usage: metrics.cpu.usage,
          cores: metrics.cpu.cores,
        },
        memory: {
          percentage: metrics.memory.percentage,
          used: metrics.memory.used,
          total: metrics.memory.total,
        },
        disk: {
          percentage: metrics.disk.percentage,
          used: metrics.disk.used,
          total: metrics.disk.total,
        },
        timestamp: Date.now(),
      };

      res.json({ metrics: publicMetrics });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  /**
   * GET /api/public/metrics/nodes - All nodes metrics summary
   */
  router.get("/metrics/nodes", (req: Request, res: Response) => {
    try {
      const nodes = nodeManager.getAllNodes();
      
      const metrics = nodes.map(node => ({
        id: node.id,
        name: node.name,
        status: node.process.status,
        blockHeight: node.metrics?.blockHeight ?? 0,
        peers: node.metrics?.connectedPeers ?? 0,
        cpuUsage: node.metrics?.cpuUsage ?? 0,
        memoryUsage: node.metrics?.memoryUsage ?? 0,
        lastUpdate: node.metrics?.lastUpdate,
      }));

      res.json({ metrics });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  return router;
}
