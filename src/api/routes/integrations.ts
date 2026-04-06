import { Router, type Request, type Response } from 'express';
import type { IntegrationManager } from '../../integrations/IntegrationManager';
import { Errors } from '../errors';
import { respondWithApiError } from '../respond';

export function createIntegrationsRouter(integrationManager: IntegrationManager): Router {
  const router = Router();

  router.get('/', (_req: Request, res: Response) => {
    try {
      const integrations = integrationManager.listAll();
      res.json({ integrations });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.get('/:id', (req: Request, res: Response) => {
    try {
      const integration = integrationManager.getOne(req.params.id as string);
      if (!integration) {
        throw Errors.notFound('Integration');
      }
      res.json({ integration });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.put('/:id', (req: Request, res: Response) => {
    try {
      const { config, enabled } = req.body as { config?: Record<string, string>; enabled?: boolean };
      if (!config || typeof enabled !== 'boolean') {
        throw Errors.missingFields('config', 'enabled');
      }
      integrationManager.saveConfig(req.params.id as string, config, enabled);
      const integration = integrationManager.getOne(req.params.id as string);
      res.json({ integration });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post('/:id/test', async (req: Request, res: Response) => {
    try {
      const result = await integrationManager.testProvider(req.params.id as string);
      res.json(result);
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.delete('/:id', (req: Request, res: Response) => {
    try {
      integrationManager.deleteConfig(req.params.id as string);
      res.status(204).send();
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}
