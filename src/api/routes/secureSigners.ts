import { Router, type Request, type Response } from "express";
import type {
  CreateSecureSignerRequest,
  UpdateSecureSignerRequest,
} from "../../types";
import type { SecureSignerManager } from "../../core/SecureSignerManager";

interface SignerParams {
  id: string;
}

export function createSecureSignersRouter(secureSignerManager: SecureSignerManager): Router {
  const router = Router();

  router.get("/", (req: Request, res: Response) => {
    try {
      const profiles = secureSignerManager.listProfiles();
      res.json({ profiles });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  router.get("/:id", (req: Request<SignerParams>, res: Response) => {
    try {
      const profile = secureSignerManager.getProfile(req.params.id);
      if (!profile) {
        return res.status(404).json({ error: "Secure signer profile not found" });
      }

      res.json({ profile });
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  router.get("/:id/orchestration", async (req: Request<SignerParams>, res: Response) => {
    try {
      const orchestration = await secureSignerManager.getOrchestration(req.params.id);
      const readiness = await secureSignerManager.getReadiness(req.params.id);
      res.json({
        orchestration: {
          ...orchestration,
          readiness,
        },
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      const status = /not found/i.test(message) ? 404 : 400;
      res.status(status).json({ error: message });
    }
  });

  router.post("/", (req: Request, res: Response) => {
    try {
      const request: CreateSecureSignerRequest = req.body;
      if (!request.name || !request.mode || !request.endpoint) {
        return res.status(400).json({ error: "Missing required fields: name, mode, endpoint" });
      }

      const profile = secureSignerManager.createProfile(request);
      res.status(201).json({ profile });
    } catch (error) {
      res.status(400).json({ error: error instanceof Error ? error.message : String(error) });
    }
  });

  router.put("/:id", (req: Request<SignerParams>, res: Response) => {
    try {
      const request: UpdateSecureSignerRequest = req.body;
      const profile = secureSignerManager.updateProfile(req.params.id, request);
      res.json({ profile });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      const status = /not found/i.test(message) ? 404 : 400;
      res.status(status).json({ error: message });
    }
  });

  router.delete("/:id", (req: Request<SignerParams>, res: Response) => {
    try {
      secureSignerManager.deleteProfile(req.params.id);
      res.status(204).send();
    } catch (error) {
      res.status(500).json({ error: error instanceof Error ? error.message : "Internal server error" });
    }
  });

  router.post("/:id/test", async (req: Request<SignerParams>, res: Response) => {
    try {
      const result = await secureSignerManager.testProfile(req.params.id);
      res.json({ result });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      const status = /not found/i.test(message) ? 404 : 400;
      res.status(status).json({ error: message });
    }
  });

  router.post("/:id/attestation", async (req: Request<SignerParams>, res: Response) => {
    try {
      const attestation = await secureSignerManager.fetchRecipientAttestation(req.params.id);
      res.json({ attestation });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      const status = /not found/i.test(message) ? 404 : 400;
      res.status(status).json({ error: message });
    }
  });

  router.post("/:id/start-recipient", async (req: Request<SignerParams>, res: Response) => {
    try {
      const ciphertextBase64 = String(req.body?.ciphertextBase64 || "").trim();
      if (!ciphertextBase64) {
        return res.status(400).json({ error: "ciphertextBase64 is required" });
      }

      const result = await secureSignerManager.startRecipientSigner(req.params.id, ciphertextBase64);
      res.json({ result });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      const status = /not found/i.test(message) ? 404 : 400;
      res.status(status).json({ error: message });
    }
  });

  return router;
}
