import { Router, type Request, type Response } from 'express';
import type { NodeManager } from '../../core/NodeManager';
import type { CreateNodeRequest, UpdateNodeRequest, ImportNodeRequest } from '../../types';

interface NodeParams {
  id: string;
}

export function createNodesRouter(nodeManager: NodeManager): Router {
  const router = Router();

  // GET /api/nodes - List all nodes
  router.get('/', (req: Request, res: Response) => {
    try {
      const nodes = nodeManager.getAllNodes();
      res.json({ nodes });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // POST /api/nodes - Create a new node
  router.post('/', async (req: Request, res: Response) => {
    try {
      const request: CreateNodeRequest = req.body;
      
      // Validate required fields
      if (!request.name || !request.type || !request.network) {
        return res.status(400).json({ 
          error: 'Missing required fields: name, type, network' 
        });
      }

      const node = await nodeManager.createNode(request);
      res.status(201).json({ node });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // POST /api/nodes/import - Import an existing node
  router.post('/import', async (req: Request, res: Response) => {
    try {
      const request: ImportNodeRequest = req.body;
      
      // Validate required fields
      if (!request.name || !request.existingPath) {
        return res.status(400).json({ 
          error: 'Missing required fields: name, existingPath' 
        });
      }

      const node = await nodeManager.importExistingNode(request);
      res.status(201).json({ node });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // POST /api/nodes/detect - Detect node configuration from path
  router.post('/detect', async (req: Request, res: Response) => {
    try {
      const { path } = req.body;
      
      if (!path) {
        return res.status(400).json({ error: 'Missing required field: path' });
      }

      const { NodeDetector } = await import('../../core/NodeDetector');
      const detected = NodeDetector.detect(path);
      
      if (!detected) {
        return res.status(404).json({ 
          error: 'No valid node installation detected at the specified path',
          path 
        });
      }

      const validation = NodeDetector.validateImport(detected);
      
      res.json({
        detected,
        validation,
        canImport: validation.valid,
      });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // POST /api/nodes/scan - Scan directory for node installations
  router.post('/scan', async (req: Request, res: Response) => {
    try {
      const { path } = req.body;
      
      if (!path) {
        return res.status(400).json({ error: 'Missing required field: path' });
      }

      const { NodeDetector } = await import('../../core/NodeDetector');
      const results = NodeDetector.scanDirectory(path);
      
      res.json({
        path,
        nodes: results,
        count: results.length,
      });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // GET /api/nodes/:id - Get node details
  router.get('/:id', (req: Request<NodeParams>, res: Response) => {
    try {
      const node = nodeManager.getNode(req.params.id);
      if (!node) {
        return res.status(404).json({ error: 'Node not found' });
      }
      res.json({ node });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // PUT /api/nodes/:id - Update node
  router.put('/:id', (req: Request<NodeParams>, res: Response) => {
    try {
      const request: UpdateNodeRequest = req.body;
      const node = nodeManager.updateNode(req.params.id, request);
      res.json({ node });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // DELETE /api/nodes/:id - Delete node
  router.delete('/:id', async (req: Request<NodeParams>, res: Response) => {
    try {
      await nodeManager.deleteNode(req.params.id);
      res.status(204).send();
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // POST /api/nodes/:id/start - Start node
  router.post('/:id/start', async (req: Request<NodeParams>, res: Response) => {
    try {
      await nodeManager.startNode(req.params.id);
      const node = nodeManager.getNode(req.params.id);
      res.json({ node });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // POST /api/nodes/:id/stop - Stop node
  router.post('/:id/stop', async (req: Request<NodeParams>, res: Response) => {
    try {
      const force = req.body?.force === true;
      await nodeManager.stopNode(req.params.id, force);
      const node = nodeManager.getNode(req.params.id);
      res.json({ node });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // POST /api/nodes/:id/restart - Restart node
  router.post('/:id/restart', async (req: Request<NodeParams>, res: Response) => {
    try {
      await nodeManager.restartNode(req.params.id);
      const node = nodeManager.getNode(req.params.id);
      res.json({ node });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // GET /api/nodes/:id/logs - Get node logs
  router.get('/:id/logs', (req: Request<NodeParams>, res: Response) => {
    try {
      const count = parseInt(req.query.count as string) || 100;
      const logs = nodeManager.getNodeLogs(req.params.id, count);
      res.json({ logs });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // GET /api/nodes/:id/storage - Get storage info
  router.get('/:id/storage', async (req: Request<NodeParams>, res: Response) => {
    try {
      const info = await nodeManager.getStorageInfo(req.params.id);
      res.json({ storage: info });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // POST /api/nodes/:id/storage/clean - Clean storage
  router.post('/:id/storage/clean', async (req: Request<NodeParams>, res: Response) => {
    try {
      const { type, maxAgeDays } = req.body;
      // Implementation for cleaning logs or chain data
      res.json({ success: true });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  return router;
}
