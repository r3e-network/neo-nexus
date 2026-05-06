import { Router, type Request, type Response } from 'express';
import type { NodeManager } from '../../core/NodeManager';
import type { MetricsCollector } from '../../monitoring/MetricsCollector';
import { enrichMetricsWithBlockHeightStatus, type NetworkHeightReader } from '../../core/blockHeightStatus';

interface NodeParams {
  id: string;
}

export function createMetricsRouter(
  nodeManager: NodeManager,
  metricsCollector: MetricsCollector,
  networkHeightTracker?: NetworkHeightReader,
): Router {
  const router = Router();

  // GET /api/metrics/system - System-wide metrics
  router.get('/system', async (req: Request, res: Response) => {
    try {
      const metrics = await metricsCollector.collectSystemMetrics();
      res.json({ metrics });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  // GET /api/nodes/:id/metrics - Node metrics
  router.get('/nodes/:id/metrics', async (req: Request<NodeParams>, res: Response) => {
    try {
      const node = nodeManager.getNode(req.params.id);
      if (!node) {
        return res.status(404).json({ error: 'Node not found' });
      }
      await nodeManager.updateMetrics(req.params.id);
      const refreshedNode = nodeManager.getNode(req.params.id);
      if (!refreshedNode) {
        return res.status(404).json({ error: 'Node not found' });
      }
      res.json({
        metrics: enrichMetricsWithBlockHeightStatus(
          refreshedNode.metrics,
          refreshedNode.network,
          networkHeightTracker,
        ),
      });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  // GET /api/nodes/:id/health - Node health check
  router.get('/nodes/:id/health', async (req: Request<NodeParams>, res: Response) => {
    try {
      const node = nodeManager.getNode(req.params.id);
      if (!node) {
        return res.status(404).json({ error: 'Node not found' });
      }

      const isHealthy = node.process.status === 'running' &&
        (node.metrics?.connectedPeers ?? 0) > 0;

      res.json({
        healthy: isHealthy,
        status: node.process.status,
        metrics: {
          blockHeight: node.metrics?.blockHeight ?? 0,
          peers: node.metrics?.connectedPeers ?? 0,
        },
      });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  // GET /api/nodes/:id/process - Process info
  router.get('/nodes/:id/process', async (req: Request<NodeParams>, res: Response) => {
    try {
      const node = nodeManager.getNode(req.params.id);
      if (!node?.process.pid) {
        return res.status(404).json({ error: 'Process not found' });
      }

      const metrics = await metricsCollector.getProcessMetrics(node.process.pid);
      res.json({ process: metrics });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  return router;
}
