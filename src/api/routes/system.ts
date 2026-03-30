import { Router, type Request, type Response } from "express";
import type { ConfigurationSnapshot } from "../../types";
import { Errors } from '../errors';
import { respondWithApiError } from '../respond';

interface SystemOperations {
  cleanOldLogs(maxAgeDays?: number): Promise<{
    cleanedFiles: number;
    nodesAffected: number;
    maxAgeDays: number;
  }>;
  exportConfiguration(): {
    generatedAt: number;
    version: string;
    nodes: unknown[];
  };
  stopAllNodes(): Promise<{
    stoppedCount: number;
    alreadyStoppedCount: number;
  }>;
  resetAllNodeData(): Promise<{
    deletedNodeCount: number;
    removedDirectoryCount: number;
    stoppedCount: number;
    alreadyStoppedCount: number;
  }>;
  restoreConfiguration(
    snapshot: ConfigurationSnapshot,
    options?: { replaceExisting?: boolean },
  ): Promise<{
    restoredCount: number;
    skippedCount: number;
    failedCount: number;
  }>;
}

export function createSystemRouter(systemOperations: SystemOperations): Router {
  const router = Router();

  router.post("/logs/clean", async (req: Request, res: Response) => {
    try {
      const requestedMaxAge = Number(req.body?.maxAgeDays);
      const maxAgeDays = Number.isFinite(requestedMaxAge) && requestedMaxAge > 0 ? requestedMaxAge : 30;
      const result = await systemOperations.cleanOldLogs(maxAgeDays);
      res.json(result);
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.get("/export", (req: Request, res: Response) => {
    try {
      const snapshot = systemOperations.exportConfiguration();
      res.setHeader("Content-Disposition", `attachment; filename="neonexus-export-${snapshot.generatedAt}.json"`);
      res.json(snapshot);
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/nodes/stop-all", async (req: Request, res: Response) => {
    try {
      const result = await systemOperations.stopAllNodes();
      res.json(result);
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/reset", async (req: Request, res: Response) => {
    try {
      const result = await systemOperations.resetAllNodeData();
      res.json(result);
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/restore", async (req: Request, res: Response) => {
    try {
      const snapshot = req.body?.snapshot as ConfigurationSnapshot | undefined;
      const replaceExisting = req.body?.replaceExisting === true;

      if (!snapshot || !Array.isArray(snapshot.nodes)) {
        throw Errors.snapshotRequired();
      }

      const result = await systemOperations.restoreConfiguration(snapshot, { replaceExisting });
      res.json(result);
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}
