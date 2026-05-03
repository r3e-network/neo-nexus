import { Router, type Request, type Response } from "express";
import type { FastSyncManager, RegisterFastSyncSnapshotInput } from "../../core/FastSyncManager";
import { respondWithApiError } from "../respond";

type FastSyncOperations = Pick<FastSyncManager, "listSnapshots" | "registerSnapshot" | "verifySnapshot">;

export function createFastSyncRouter(fastSyncManager: FastSyncOperations): Router {
  const router = Router();

  router.get("/snapshots", (_req: Request, res: Response) => {
    try {
      const snapshots = fastSyncManager.listSnapshots();
      res.json({ snapshots });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/snapshots", (req: Request, res: Response) => {
    try {
      const snapshot = fastSyncManager.registerSnapshot(req.body as RegisterFastSyncSnapshotInput);
      res.status(201).json({ snapshot });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/snapshots/:id/verify", async (req: Request, res: Response) => {
    try {
      const snapshot = await fastSyncManager.verifySnapshot(req.params.id as string);
      res.json({ snapshot });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}
