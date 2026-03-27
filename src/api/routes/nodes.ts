import { Router, type Request, type Response } from 'express';
import { resolve } from 'node:path';
import type { NodeManager } from '../../core/NodeManager';
import type { CreateNodeRequest, UpdateNodeRequest, ImportNodeRequest } from '../../types';
import { paths } from '../../utils/paths';

function validateNodePath(inputPath: string): string {
  const resolved = resolve(inputPath);
  const allowedPrefixes = [paths.nodes, paths.base, '/home', '/opt', '/var/lib'];
  const blocked = ['/', '/etc', '/root', '/proc', '/sys', '/dev'];

  if (blocked.includes(resolved)) {
    throw new Error(`Access to path ${resolved} is not permitted`);
  }

  if (!allowedPrefixes.some((prefix) => resolved.startsWith(prefix))) {
    throw new Error(`Path must be under an allowed directory`);
  }

  return resolved;
}

interface NodeParams {
  id: string;
}

export function createNodesRouter(nodeManager: NodeManager): Router {
  const router = Router();

  const respondWithNodeError = (res: Response, error: unknown) => {
    const message = error instanceof Error ? error.message : String(error);

    if (/not found/i.test(message)) {
      return res.status(404).json({ error: message });
    }

    if (
      /missing required fields|invalid|already exists|requires|cannot|not available|unsupported/i.test(
        message,
      )
    ) {
      return res.status(400).json({ error: message });
    }

    return res.status(500).json({ error: message });
  };

  const validateSecureSignerRequest = (
    request: Pick<CreateNodeRequest, "type" | "settings"> | Pick<UpdateNodeRequest, "settings">,
    existingNodeType?: CreateNodeRequest["type"],
  ): string | null => {
    const keyProtection = request.settings?.keyProtection;
    if (keyProtection?.mode !== "secure-signer") {
      return null;
    }

    const nodeType = "type" in request && request.type ? request.type : existingNodeType;
    if (nodeType !== "neo-cli") {
      return "Secure signer protection currently requires a neo-cli node with SignClient support";
    }

    if (!keyProtection.signerProfileId) {
      return "Secure signer protection requires a signer profile";
    }

    const profile = nodeManager.getSecureSignerManager().getProfile(keyProtection.signerProfileId);
    if (!profile || !profile.enabled) {
      return `Secure signer profile ${keyProtection.signerProfileId} is not available`;
    }

    return null;
  };

  // GET /api/nodes - List all nodes
  router.get('/', (_req: Request, res: Response) => {
    try {
      const nodes = nodeManager.getAllNodes();
      res.json({ nodes });
    } catch (error) {
      respondWithNodeError(res, error);
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

      const secureSignerValidationError = validateSecureSignerRequest(request);
      if (secureSignerValidationError) {
        return res.status(400).json({ error: secureSignerValidationError });
      }

      const node = await nodeManager.createNode(request);
      res.status(201).json({ node });
    } catch (error) {
      respondWithNodeError(res, error);
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

      request.existingPath = validateNodePath(request.existingPath);

      const node = await nodeManager.importExistingNode(request);
      res.status(201).json({ node });
    } catch (error) {
      respondWithNodeError(res, error);
    }
  });

  // POST /api/nodes/detect - Detect node configuration from path
  router.post('/detect', async (req: Request, res: Response) => {
    try {
      const { path } = req.body;

      if (!path) {
        return res.status(400).json({ error: 'Missing required field: path' });
      }

      const validatedPath = validateNodePath(path);
      const { NodeDetector } = await import('../../core/NodeDetector');
      const detected = NodeDetector.detect(validatedPath);
      
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
      respondWithNodeError(res, error);
    }
  });

  // POST /api/nodes/scan - Scan directory for node installations
  router.post('/scan', async (req: Request, res: Response) => {
    try {
      const { path } = req.body;

      if (!path) {
        return res.status(400).json({ error: 'Missing required field: path' });
      }

      const validatedPath = validateNodePath(path);
      const { NodeDetector } = await import('../../core/NodeDetector');
      const results = NodeDetector.scanDirectory(validatedPath);

      res.json({
        path: validatedPath,
        nodes: results,
        count: results.length,
      });
    } catch (error) {
      respondWithNodeError(res, error);
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
      respondWithNodeError(res, error);
    }
  });

  // PUT /api/nodes/:id - Update node
  router.put('/:id', async (req: Request<NodeParams>, res: Response) => {
    try {
      const request: UpdateNodeRequest = req.body;
      const existingNode = nodeManager.getNode(req.params.id);
      if (!existingNode) {
        return res.status(404).json({ error: "Node not found" });
      }

      const secureSignerValidationError = validateSecureSignerRequest(request, existingNode.type);
      if (secureSignerValidationError) {
        return res.status(400).json({ error: secureSignerValidationError });
      }
      const keyProtection = request.settings?.keyProtection;

      const node = await nodeManager.updateNode(req.params.id, request);

      if (keyProtection?.mode === "secure-signer") {
        await nodeManager.syncNodeSecureSigner(req.params.id);
      }

      res.json({ node });
    } catch (error) {
      respondWithNodeError(res, error);
    }
  });

  // DELETE /api/nodes/:id - Delete node
  router.delete('/:id', async (req: Request<NodeParams>, res: Response) => {
    try {
      await nodeManager.deleteNode(req.params.id);
      res.status(204).send();
    } catch (error) {
      respondWithNodeError(res, error);
    }
  });

  // POST /api/nodes/:id/start - Start node
  router.post('/:id/start', async (req: Request<NodeParams>, res: Response) => {
    try {
      await nodeManager.startNode(req.params.id);
      const node = nodeManager.getNode(req.params.id);
      res.json({ node });
    } catch (error) {
      respondWithNodeError(res, error);
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
      respondWithNodeError(res, error);
    }
  });

  // POST /api/nodes/:id/restart - Restart node
  router.post('/:id/restart', async (req: Request<NodeParams>, res: Response) => {
    try {
      await nodeManager.restartNode(req.params.id);
      const node = nodeManager.getNode(req.params.id);
      res.json({ node });
    } catch (error) {
      respondWithNodeError(res, error);
    }
  });

  // GET /api/nodes/:id/logs - Get node logs
  router.get('/:id/logs', (req: Request<NodeParams>, res: Response) => {
    try {
      const count = Math.min(parseInt(req.query.count as string) || 100, 1000);
      const logs = nodeManager.getNodeLogs(req.params.id, count);
      res.json({ logs });
    } catch (error) {
      respondWithNodeError(res, error);
    }
  });

  // GET /api/nodes/:id/storage - Get storage info
  router.get('/:id/storage', async (req: Request<NodeParams>, res: Response) => {
    try {
      const info = await nodeManager.getStorageInfo(req.params.id);
      res.json({ storage: info });
    } catch (error) {
      respondWithNodeError(res, error);
    }
  });

  // GET /api/nodes/:id/signer-health - Get bound secure signer readiness
  router.get('/:id/signer-health', async (req: Request<NodeParams>, res: Response) => {
    try {
      const signerHealth = await nodeManager.getNodeSecureSignerHealth(req.params.id);
      if (!signerHealth) {
        return res.json({ signerHealth: null });
      }
      res.json({ signerHealth });
    } catch (error) {
      respondWithNodeError(res, error);
    }
  });

  // POST /api/nodes/:id/storage/clean - Clean storage (logs)
  router.post('/:id/storage/clean', async (req: Request<NodeParams>, res: Response) => {
    try {
      const node = nodeManager.getNode(req.params.id);
      if (!node) {
        return res.status(404).json({ error: 'Node not found' });
      }
      const requestedMaxAge = Number(req.body?.maxAgeDays);
      const maxAgeDays = Number.isFinite(requestedMaxAge) && requestedMaxAge > 0 ? requestedMaxAge : 30;
      const { StorageManager } = await import('../../core/StorageManager');
      const cleanedFiles = await StorageManager.cleanOldLogs(node.paths.logs, maxAgeDays);
      res.json({ success: true, cleanedFiles, maxAgeDays });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : String(error) });
    }
  });

  return router;
}
