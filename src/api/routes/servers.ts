import { Router, type Request, type Response } from "express";
import type { CreateRemoteServerRequest, RemoteServerProfile, RemoteServerSummary, UpdateRemoteServerRequest } from "../../types";
import type { AuthenticatedRequest } from "../middleware/auth";
import { remoteServerSummariesResponseForRole, remoteServerSummaryResponseForRole } from "../serializers/remoteServerResponses";
import { ApiError, Errors } from '../errors';
import { respondWithApiError } from '../respond';

interface ServerProfilesOperations {
  listServersWithStatus(): Promise<RemoteServerSummary[]>;
  getServerSummary(id: string): Promise<RemoteServerSummary>;
  createServer(request: CreateRemoteServerRequest): RemoteServerProfile;
  updateServer(id: string, request: UpdateRemoteServerRequest): RemoteServerProfile;
  deleteServer(id: string): void;
}

type MaybeAuthenticatedRequest<P = Record<string, string>> = Request<P> & {
  user?: AuthenticatedRequest["user"];
};

export function createServersRouter(serverManager: ServerProfilesOperations): Router {
  const router = Router();

  const validateRemoteServerBaseUrl = (baseUrl: unknown): void => {
    if (typeof baseUrl !== "string" || baseUrl.trim() === "") {
      throw Errors.missingField('baseUrl (must be a valid URL)');
    }

    try {
      const parsed = new URL(baseUrl);
      if (!['http:', 'https:'].includes(parsed.protocol)) {
        throw Errors.serverUrlProtocolInvalid();
      }
    } catch (error) {
      if (error instanceof ApiError) throw error;
      throw Errors.missingField('baseUrl (must be a valid URL)');
    }
  };

  router.get("/", async (req: MaybeAuthenticatedRequest, res: Response) => {
    try {
      const servers = await serverManager.listServersWithStatus();
      res.json({ servers: remoteServerSummariesResponseForRole(req.user?.role, servers) });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.get("/:id", async (req: MaybeAuthenticatedRequest<{ id: string }>, res: Response) => {
    try {
      const server = await serverManager.getServerSummary(req.params.id as string);
      res.json({ server: remoteServerSummaryResponseForRole(req.user?.role, server) });
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
      validateRemoteServerBaseUrl(baseUrl);
      const server = serverManager.createServer(req.body as CreateRemoteServerRequest);
      res.status(201).json({ server });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.put("/:id", (req: Request, res: Response) => {
    try {
      if (req.body?.baseUrl !== undefined) {
        validateRemoteServerBaseUrl(req.body.baseUrl);
      }
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
