import { Router, type Request, type Response } from "express";
import type { PrivateNetworkManager, CreatePrivateNetworkPlanInput } from "../../core/PrivateNetworkManager";
import type { ConfigurationSnapshot } from "../../types";
import { ApiError } from "../errors";
import { respondWithApiError } from "../respond";

type PrivateNetworkOperations = Pick<
  PrivateNetworkManager,
  "listPlans" | "getPlan" | "createPlan" | "buildConfigurationSnapshot" | "markApplied"
>;

interface RestoreOperations {
  restoreConfiguration(
    snapshot: ConfigurationSnapshot,
    options?: { replaceExisting?: boolean },
  ): Promise<{
    restoredCount: number;
    skippedCount: number;
    failedCount: number;
  }>;
}

export function createPrivateNetworksRouter(
  privateNetworkManager: PrivateNetworkOperations,
  nodeManager: RestoreOperations,
): Router {
  const router = Router();

  router.get("/plans", (_req: Request, res: Response) => {
    try {
      res.json({ plans: privateNetworkManager.listPlans() });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/plans", (req: Request, res: Response) => {
    try {
      const plan = privateNetworkManager.createPlan(req.body as CreatePrivateNetworkPlanInput);
      res.status(201).json({ plan });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });
  router.post("/plan", (req: Request, res: Response) => {
    try {
      const plan = privateNetworkManager.createPlan(req.body as CreatePrivateNetworkPlanInput);
      res.status(201).json({ plan });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.get("/plans/:id", (req: Request, res: Response) => {
    try {
      const planId = String(req.params.id);
      const plan = privateNetworkManager.getPlan(planId, { required: true });
      res.json({ plan });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.get("/plans/:id/configuration-snapshot", (req: Request, res: Response) => {
    try {
      const planId = String(req.params.id);
      const snapshot = privateNetworkManager.buildConfigurationSnapshot(planId);
      res.json({ snapshot });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/plans/:id/apply", async (req: Request, res: Response) => {
    try {
      await applyPlan(req, res, String(req.params.id));
    } catch (error) {
      respondWithApiError(res, error);
    }
  });
  router.post("/:planId/apply", async (req: Request, res: Response) => {
    try {
      await applyPlan(req, res, String(req.params.planId));
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;

  async function applyPlan(req: Request, res: Response, planId: string): Promise<void> {
    const snapshot = privateNetworkManager.buildConfigurationSnapshot(planId);
    const replaceExisting = parseReplaceExisting(req.body?.replaceExisting);
    const result = await nodeManager.restoreConfiguration(snapshot, { replaceExisting });
    const expectedCount = snapshot.nodes.length;
    const fullyRestored = result.restoredCount === expectedCount && result.skippedCount === 0 && result.failedCount === 0;
    const plan = fullyRestored ? privateNetworkManager.markApplied(planId) : null;
    res.json({ result, plan });
  }
}

function parseReplaceExisting(value: unknown): boolean {
  if (value === undefined || value === null) {
    return false;
  }
  if (typeof value !== "boolean") {
    throw new ApiError(
      "PRIVATE_NETWORK_REPLACE_EXISTING_INVALID",
      "replaceExisting must be a boolean",
      "Use true only when you intentionally want to replace all existing managed nodes.",
    );
  }
  return value;
}
