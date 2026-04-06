import { Router, type Request, type Response } from "express";
import type { CreateRemoteServerRequest, UpdateRemoteServerRequest } from "../../types";
import { ApiError, Errors } from '../errors';
import { respondWithApiError } from '../respond';

interface ServerProfilesOperations {
  listServersWithStatus(): Promise<unknown[]>;
  getServerSummary(id: string): Promise<unknown>;
  createServer(request: CreateRemoteServerRequest): unknown;
  updateServer(id: string, request: UpdateRemoteServerRequest): unknown;
  deleteServer(id: string): void;
}

export function createServersRouter(serverManager: ServerProfilesOperations): Router {
  const router = Router();

  router.get("/", async (_req: Request, res: Response) => {
    try {
      const servers = await serverManager.listServersWithStatus();
      res.json({ servers });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.get("/:id", async (req: Request, res: Response) => {
    try {
      const server = await serverManager.getServerSummary(req.params.id as string);
      res.json({ server });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/", (req: Request, res: Response) => {
    try {
      const { name, baseUrl } = req.body || {};
      if (!name || !baseUrl) {
        throw Errors.serverFieldsRequired();
      }
      try {
        const parsed = new URL(baseUrl);
        if (!['http:', 'https:'].includes(parsed.protocol)) {
          throw Errors.missingField('baseUrl (must use http or https)');
        }
      } catch (e) {
        if (e instanceof ApiError) throw e;
        throw Errors.missingField('baseUrl (must be a valid URL)');
      }
      const server = serverManager.createServer(req.body as CreateRemoteServerRequest);
      res.status(201).json({ server });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.put("/:id", (req: Request, res: Response) => {
    try {
      const server = serverManager.updateServer(req.params.id as string, req.body as UpdateRemoteServerRequest);
      res.json({ server });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.delete("/:id", (req: Request, res: Response) => {
    try {
      serverManager.deleteServer(req.params.id as string);
      res.status(204).send();
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}
