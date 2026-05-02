import { Router, type Request, type Response } from 'express';
import { isAbsolute, relative, resolve } from 'node:path';
import type { NodeManager } from '../../core/NodeManager';
import { ConfigManager } from '../../core/ConfigManager';
import type { CreateNodeRequest, UpdateNodeRequest, ImportNodeRequest, ImportedNodeOwnershipMode, NodeInstance, NodeSettings } from '../../types';
import { paths } from '../../utils/paths';
import { assertNodeNetwork, assertNodeType } from '../../utils/nodeValidation';
import { ApiError, Errors } from '../errors';
import { respondWithApiError } from '../respond';
import type { AuthenticatedRequest } from '../middleware/auth';

function validateNodePath(inputPath: string): string {
  const resolved = resolve(inputPath);
  const allowedPrefixes = [paths.nodes, paths.base, '/home', '/opt', '/var/lib'];
  const blocked = ['/', '/etc', '/root', '/proc', '/sys', '/dev'];

  if (blocked.includes(resolved)) {
    throw Errors.pathBlocked(resolved);
  }

  if (!allowedPrefixes.some((prefix) => isPathWithinOrEqual(resolved, prefix))) {
    throw Errors.pathNotAllowed();
  }

  return resolved;
}

function isPathWithinOrEqual(resolvedPath: string, allowedPrefix: string): boolean {
  const resolvedPrefix = resolve(allowedPrefix);
  const pathRelativeToPrefix = relative(resolvedPrefix, resolvedPath);
  return pathRelativeToPrefix === '' || (!pathRelativeToPrefix.startsWith('..') && !isAbsolute(pathRelativeToPrefix));
}

interface NodeParams {
  id: string;
}

type MaybeAuthenticatedRequest = {
  params: unknown;
  user?: AuthenticatedRequest['user'];
};

function isViewerRequest(req: MaybeAuthenticatedRequest): boolean {
  return req.user?.role === 'viewer';
}

function requireAdminRead(req: MaybeAuthenticatedRequest): void {
  if (req.user && req.user.role !== 'admin') {
    throw Errors.adminRequired();
  }
}

function sanitizeNodeSettingsForViewer(settings: NodeSettings): Partial<NodeSettings> {
  return {
    ...(settings.maxConnections !== undefined ? { maxConnections: settings.maxConnections } : {}),
    ...(settings.minPeers !== undefined ? { minPeers: settings.minPeers } : {}),
    ...(settings.maxPeers !== undefined ? { maxPeers: settings.maxPeers } : {}),
    ...(settings.relay !== undefined ? { relay: settings.relay } : {}),
    ...(settings.debugMode !== undefined ? { debugMode: settings.debugMode } : {}),
    ...(settings.resourceLimits ? { resourceLimits: settings.resourceLimits } : {}),
    ...(settings.keyProtection ? { keyProtection: { mode: settings.keyProtection.mode } } : {}),
    ...(settings.import ? {
      import: {
        imported: settings.import.imported,
        ownershipMode: settings.import.ownershipMode,
      },
    } : {}),
  };
}

function sanitizeNodeForViewer(node: NodeInstance): Omit<NodeInstance, 'paths' | 'settings' | 'plugins'> & {
  settings: Partial<NodeSettings>;
  plugins?: Array<Pick<NonNullable<NodeInstance['plugins']>[number], 'id' | 'version' | 'installedAt' | 'enabled'>>;
} {
  const { paths: _paths, settings, plugins, ...safeNode } = node;
  return {
    ...safeNode,
    settings: sanitizeNodeSettingsForViewer(settings),
    plugins: plugins?.map(({ id, version, installedAt, enabled }) => ({ id, version, installedAt, enabled })),
  };
}

function nodeResponseForRequest(req: MaybeAuthenticatedRequest, node: NodeInstance): NodeInstance | ReturnType<typeof sanitizeNodeForViewer> {
  return isViewerRequest(req) ? sanitizeNodeForViewer(node) : node;
}

function storageResponseForRequest<T extends { chain?: { path?: string }; wallets?: { path?: string } }>(
  req: MaybeAuthenticatedRequest,
  storage: T,
): T | Omit<T, 'chain' | 'wallets'> & {
  chain?: Omit<NonNullable<T['chain']>, 'path'>;
  wallets?: Omit<NonNullable<T['wallets']>, 'path'>;
} {
  if (!isViewerRequest(req)) {
    return storage;
  }
  const { chain, wallets, ...safeStorage } = storage;
  return {
    ...safeStorage,
    ...(chain ? { chain: omitPath(chain) } : {}),
    ...(wallets ? { wallets: omitPath(wallets) } : {}),
  };
}

function signerHealthResponseForRequest<
  T extends {
    profile?: {
      id?: string;
      name: string;
      mode: string;
      endpoint?: string;
    };
  },
>(req: MaybeAuthenticatedRequest, signerHealth: T): T | Omit<T, 'profile'> & {
  profile?: Pick<NonNullable<T['profile']>, 'name' | 'mode'>;
} {
  if (!isViewerRequest(req) || !signerHealth.profile) {
    return signerHealth;
  }
  const { profile, ...safeSignerHealth } = signerHealth;
  return {
    ...safeSignerHealth,
    profile: {
      name: profile.name,
      mode: profile.mode,
    },
  };
}

function omitPath<T extends { path?: string }>(value: T): Omit<T, 'path'> {
  const { path: _path, ...safeValue } = value;
  return safeValue;
}

export function createNodesRouter(nodeManager: NodeManager): Router {
  const router = Router();

  const validateOwnershipMode = (mode: unknown): void => {
    if (mode === undefined) {
      return;
    }
    if (mode !== 'observe-only' && mode !== 'managed-config' && mode !== 'managed-process') {
      throw new ApiError(
        'INVALID_OWNERSHIP_MODE',
        'Invalid import ownership mode',
        'Use observe-only, managed-config, or managed-process for imported native nodes.',
      );
    }
  };

  const validateSecureSignerRequest = (
    request: Pick<CreateNodeRequest, "type" | "settings"> | Pick<UpdateNodeRequest, "settings">,
    existingNodeType?: CreateNodeRequest["type"],
  ): ApiError | null => {
    const keyProtection = request.settings?.keyProtection;
    if (keyProtection?.mode !== "secure-signer") {
      return null;
    }

    const nodeType = "type" in request && request.type ? request.type : existingNodeType;
    if (nodeType !== "neo-cli") {
      return Errors.signerNeoCliOnly();
    }

    if (!keyProtection.signerProfileId) {
      return Errors.signerRequiresProfile();
    }

    const profile = nodeManager.getSecureSignerManager().getProfile(keyProtection.signerProfileId);
    if (!profile || !profile.enabled) {
      return Errors.signerNotAvailable(keyProtection.signerProfileId);
    }

    return null;
  };

  // GET /api/nodes - List all nodes
  router.get('/', (_req: Request, res: Response) => {
    try {
      const nodes = nodeManager.getAllNodes();
      res.json({ nodes: nodes.map((node) => nodeResponseForRequest(_req, node)) });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // POST /api/nodes - Create a new node
  router.post('/', async (req: Request, res: Response) => {
    try {
      const request: CreateNodeRequest = req.body;

      // Validate required fields
      if (!request.name || !request.type || !request.network) {
        throw Errors.missingFields("name", "type", "network");
      }
      assertNodeType(request.type);
      assertNodeNetwork(request.network);

      const secureSignerValidationError = validateSecureSignerRequest(request);
      if (secureSignerValidationError) {
        throw secureSignerValidationError;
      }

      const node = await nodeManager.createNode(request);
      res.status(201).json({ node });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // POST /api/nodes/import - Import an existing node
  router.post('/import', async (req: Request, res: Response) => {
    try {
      const request: ImportNodeRequest = req.body;

      // Validate required fields
      if (!request.name || !request.existingPath) {
        throw Errors.missingFields("name", "existingPath");
      }

      validateOwnershipMode(request.ownershipMode);
      const validatedRequest = { ...request, existingPath: validateNodePath(request.existingPath) };

      const node = await nodeManager.importExistingNode(validatedRequest);
      res.status(201).json({ node });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // POST /api/nodes/detect - Detect node configuration from path
  router.post('/detect', async (req: Request, res: Response) => {
    try {
      const { path } = req.body;

      if (!path) {
        throw Errors.missingField("path");
      }

      const validatedPath = validateNodePath(path);
      const { NodeDetector } = await import('../../core/NodeDetector');
      const detected = NodeDetector.detect(validatedPath);

      if (!detected) {
        throw Errors.detectionNotFound();
      }

      const validation = NodeDetector.validateImport(detected);

      res.json({
        detected,
        validation,
        canImport: validation.valid,
      });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // POST /api/nodes/scan - Scan directory for node installations
  router.post('/scan', async (req: Request, res: Response) => {
    try {
      const { path } = req.body;

      if (!path) {
        throw Errors.missingField("path");
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
      respondWithApiError(res, error);
    }
  });

  // GET /api/nodes/:id - Get node details
  router.get('/:id', (req: Request<NodeParams>, res: Response) => {
    try {
      const node = nodeManager.getNode(req.params.id);
      if (!node) {
        throw Errors.nodeNotFound(req.params.id);
      }
      res.json({ node: nodeResponseForRequest(req, node) });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // PUT /api/nodes/:id - Update node
  router.put('/:id', async (req: Request<NodeParams>, res: Response) => {
    try {
      const request: UpdateNodeRequest = req.body;
      const existingNode = nodeManager.getNode(req.params.id);
      if (!existingNode) {
        throw Errors.nodeNotFound(req.params.id);
      }

      const secureSignerValidationError = validateSecureSignerRequest(request, existingNode.type);
      if (secureSignerValidationError) {
        throw secureSignerValidationError;
      }
      const node = await nodeManager.updateNode(req.params.id, request);

      res.json({ node });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // DELETE /api/nodes/:id - Delete node
  router.delete('/:id', async (req: Request<NodeParams>, res: Response) => {
    try {
      await nodeManager.deleteNode(req.params.id);
      res.status(204).send();
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // POST /api/nodes/:id/ownership - Explicitly adopt or downgrade imported native node ownership
  router.post('/:id/ownership', async (req: Request<NodeParams>, res: Response) => {
    try {
      const ownershipMode = req.body?.ownershipMode as ImportedNodeOwnershipMode | undefined;
      validateOwnershipMode(ownershipMode);
      if (!ownershipMode) {
        throw Errors.missingField('ownershipMode');
      }

      const node = await nodeManager.updateImportedNodeOwnership(req.params.id, ownershipMode);
      res.json({ node });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // POST /api/nodes/:id/start - Start node
  router.post('/:id/start', async (req: Request<NodeParams>, res: Response) => {
    try {
      await nodeManager.startNode(req.params.id);
      const node = nodeManager.getNode(req.params.id);
      res.json({ node });
    } catch (error) {
      respondWithApiError(res, error);
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
      respondWithApiError(res, error);
    }
  });

  // POST /api/nodes/:id/restart - Restart node
  router.post('/:id/restart', async (req: Request<NodeParams>, res: Response) => {
    try {
      await nodeManager.restartNode(req.params.id);
      const node = nodeManager.getNode(req.params.id);
      res.json({ node });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // GET /api/nodes/:id/logs - Get node logs
  router.get('/:id/logs', (req: Request<NodeParams>, res: Response) => {
    try {
      const node = nodeManager.getNode(req.params.id);
      if (!node) {
        throw Errors.nodeNotFound(req.params.id);
      }
      const count = Math.max(1, Math.min(parseInt(req.query.count as string) || 100, 1000));
      const logs = nodeManager.getNodeLogs(req.params.id, count);
      res.json({ logs });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // GET /api/nodes/:id/storage - Get storage info
  router.get('/:id/storage', async (req: Request<NodeParams>, res: Response) => {
    try {
      const info = await nodeManager.getStorageInfo(req.params.id);
      res.json({ storage: storageResponseForRequest(req, info) });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // GET /api/nodes/:id/signer-health - Get bound secure signer readiness
  router.get('/:id/signer-health', async (req: Request<NodeParams>, res: Response) => {
    try {
      const node = nodeManager.getNode(req.params.id);
      if (!node) {
        throw Errors.nodeNotFound(req.params.id);
      }
      const signerHealth = await nodeManager.getNodeSecureSignerHealth(req.params.id);
      if (!signerHealth) {
        return res.json({ signerHealth: null });
      }
      res.json({ signerHealth: signerHealthResponseForRequest(req, signerHealth) });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // POST /api/nodes/:id/storage/clean - Clean storage (logs)
  router.post('/:id/storage/clean', async (req: Request<NodeParams>, res: Response) => {
    try {
      const node = nodeManager.getNode(req.params.id);
      if (!node) {
        throw Errors.nodeNotFound(req.params.id);
      }
      const requestedMaxAge = Number(req.body?.maxAgeDays);
      const maxAgeDays = Number.isFinite(requestedMaxAge) && requestedMaxAge > 0 ? requestedMaxAge : 30;
      const cleanedFiles = await nodeManager.cleanNodeLogs(req.params.id, maxAgeDays);
      res.json({ success: true, cleanedFiles, maxAgeDays });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  // GET /api/nodes/:id/config-audit - Audit node configuration
  router.get('/:id/config-audit', async (req: Request<NodeParams>, res: Response) => {
    try {
      requireAdminRead(req);
      const node = nodeManager.getNode(req.params.id);
      if (!node) {
        throw Errors.nodeNotFound(req.params.id);
      }
      const plugins = nodeManager.getPluginManager().getInstalledPlugins(req.params.id).map(p => p.id);
      const audit = await ConfigManager.auditNodeConfig(node, plugins);
      res.json({ audit });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}
