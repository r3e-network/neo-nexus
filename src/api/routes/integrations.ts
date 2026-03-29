import { Router, type Request, type Response } from 'express';
import type { IntegrationManager } from '../../integrations/IntegrationManager';

export function createIntegrationsRouter(integrationManager: IntegrationManager): Router {
  const router = Router();

  router.get('/', (_req: Request, res: Response) => {
    try {
      const integrations = integrationManager.listAll();
      res.json({ integrations });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : 'Internal server error' });
    }
  });

  router.get('/:id', (req: Request, res: Response) => {
    try {
      const integration = integrationManager.getOne(req.params.id as string);
      if (!integration) {
        return res.status(404).json({ error: 'Integration not found' });
      }
      res.json({ integration });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : 'Internal server error' });
    }
  });

  router.put('/:id', (req: Request, res: Response) => {
    try {
      const { config, enabled } = req.body as { config?: Record<string, string>; enabled?: boolean };
      if (!config || typeof enabled !== 'boolean') {
        return res.status(400).json({ error: 'Missing required fields: config (object), enabled (boolean)' });
      }
      integrationManager.saveConfig(req.params.id as string, config, enabled);
      const integration = integrationManager.getOne(req.params.id as string);
      res.json({ integration });
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Bad request';
      const status = msg.includes('Unknown integration') ? 404 : 400;
      res.status(status).json({ error: msg });
    }
  });

  router.post('/:id/test', async (req: Request, res: Response) => {
    try {
      const result = await integrationManager.testProvider(req.params.id as string);
      res.json(result);
    } catch (error) {
      res.status(500).json({ success: false, error: error instanceof Error ? error.message : 'Test failed' });
    }
  });

  router.delete('/:id', (req: Request, res: Response) => {
    try {
      integrationManager.deleteConfig(req.params.id as string);
      res.status(204).send();
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Bad request';
      const status = msg.includes('Unknown integration') ? 404 : 400;
      res.status(status).json({ error: msg });
    }
  });

  return router;
}
