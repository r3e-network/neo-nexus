import { Router, type Request, type Response } from 'express';
import type { NodeManager } from '../../core/NodeManager';
import type { PluginId } from '../../types';

interface NodeParams {
  id: string;
  pluginId: string;
}

export function createPluginsRouter(nodeManager: NodeManager): Router {
  const router = Router({ mergeParams: true });
  const pluginManager = nodeManager.getPluginManager();

  // GET /api/nodes/:id/plugins - List installed plugins
  router.get('/', (req: Request<NodeParams>, res: Response) => {
    try {
      const plugins = pluginManager.getInstalledPlugins(req.params.id);
      res.json({ plugins });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // POST /api/nodes/:id/plugins - Install plugin
  router.post('/', async (req: Request<NodeParams>, res: Response) => {
    try {
      const { pluginId, config } = req.body;
      
      if (!pluginId) {
        return res.status(400).json({ error: 'pluginId is required' });
      }

      await nodeManager.installPlugin(req.params.id, pluginId as PluginId, config);
      const plugins = pluginManager.getInstalledPlugins(req.params.id);
      res.status(201).json({ plugins });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // GET /api/nodes/:id/plugins/available - List available plugins
  router.get('/available', (req: Request<NodeParams>, res: Response) => {
    try {
      const plugins = pluginManager.getAvailablePlugins();
      res.json({ plugins });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // PUT /api/nodes/:id/plugins/:pluginId - Update plugin config
  router.put('/:pluginId', (req: Request<NodeParams>, res: Response) => {
    try {
      const { config } = req.body;
      pluginManager.updatePluginConfig(req.params.id, req.params.pluginId as PluginId, config);
      const plugins = pluginManager.getInstalledPlugins(req.params.id);
      res.json({ plugins });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // DELETE /api/nodes/:id/plugins/:pluginId - Uninstall plugin
  router.delete('/:pluginId', async (req: Request<NodeParams>, res: Response) => {
    try {
      await pluginManager.uninstallPlugin(req.params.id, req.params.pluginId as PluginId);
      res.status(204).send();
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // POST /api/nodes/:id/plugins/:pluginId/enable - Enable plugin
  router.post('/:pluginId/enable', (req: Request<NodeParams>, res: Response) => {
    try {
      pluginManager.setPluginEnabled(req.params.id, req.params.pluginId as PluginId, true);
      const plugins = pluginManager.getInstalledPlugins(req.params.id);
      res.json({ plugins });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  // POST /api/nodes/:id/plugins/:pluginId/disable - Disable plugin
  router.post('/:pluginId/disable', (req: Request<NodeParams>, res: Response) => {
    try {
      pluginManager.setPluginEnabled(req.params.id, req.params.pluginId as PluginId, false);
      const plugins = pluginManager.getInstalledPlugins(req.params.id);
      res.json({ plugins });
    } catch (error) {
      res.status(500).json({ error: String(error) });
    }
  });

  return router;
}
