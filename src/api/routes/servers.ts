import { Router, type Request, type Response } from "express";
import type { CreateRemoteServerRequest, UpdateRemoteServerRequest } from "../../types";

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
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  router.get("/:id", async (req: Request, res: Response) => {
    try {
      const server = await serverManager.getServerSummary(req.params.id as string);
      res.json({ server });
    } catch (error) {
      res.status(404).json({ error: error instanceof Error ? error.message : "Not found" });
    }
  });

  router.post("/", (req: Request, res: Response) => {
    try {
      const { name, baseUrl } = req.body || {};
      if (!name || !baseUrl) {
        return res.status(400).json({ error: "Missing required fields: name, baseUrl" });
      }
      const server = serverManager.createServer(req.body as CreateRemoteServerRequest);
      res.status(201).json({ server });
    } catch (error) {
      res.status(400).json({ error: error instanceof Error ? error.message : "Bad request" });
    }
  });

  router.put("/:id", (req: Request, res: Response) => {
    try {
      const server = serverManager.updateServer(req.params.id as string, req.body as UpdateRemoteServerRequest);
      res.json({ server });
    } catch (error) {
      res.status(400).json({ error: error instanceof Error ? error.message : "Bad request" });
    }
  });

  router.delete("/:id", (req: Request, res: Response) => {
    try {
      serverManager.deleteServer(req.params.id as string);
      res.status(204).send();
    } catch (error) {
      res.status(400).json({ error: error instanceof Error ? error.message : "Bad request" });
    }
  });

  return router;
}
