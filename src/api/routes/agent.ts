import { Router, type Request, type Response } from "express";
import type { AgentManager } from "../../agent/AgentManager";
import type { AuthenticatedRequest } from "../middleware/auth";
import { respondWithApiError } from "../respond";
import { Errors } from "../errors";

interface SettingsBody {
  provider?: string;
  model?: string;
  apiKey?: string;
  baseUrl?: string;
  enabled?: boolean;
}

interface ConversationBody {
  title?: string;
}

interface SendBody {
  text?: string;
}

const SECRET_PREFIX = "••••••••...";

export function createAgentRouter(agent: AgentManager): Router {
  const router = Router();

  router.get("/health", (_req: Request, res: Response) => {
    res.json({ enabled: agent.isEnabled() });
  });

  router.get("/settings", (req: Request, res: Response) => {
    const user = (req as AuthenticatedRequest).user;
    const settings = agent.getSettings(user.id);
    if (!settings) {
      res.json({ configured: false });
      return;
    }
    res.json({
      configured: true,
      provider: settings.provider,
      model: settings.model,
      baseUrl: settings.baseUrl ?? null,
      enabled: settings.enabled,
      apiKey: redact(settings.apiKey),
      updatedAt: settings.updatedAt,
    });
  });

  router.put("/settings", (req: Request<unknown, unknown, SettingsBody>, res: Response) => {
    const user = (req as AuthenticatedRequest).user;
    const body = req.body ?? {};

    const provider = body.provider;
    if (provider !== "anthropic" && provider !== "openai" && provider !== "openai-compatible") {
      respondWithApiError(res, Errors.missingFields("provider"));
      return;
    }
    if (!body.model || !body.apiKey) {
      respondWithApiError(res, Errors.missingFields(...[!body.model ? "model" : null, !body.apiKey ? "apiKey" : null].filter((x): x is string => Boolean(x))));
      return;
    }

    try {
      const saved = agent.saveSettings(user.id, {
        provider,
        model: body.model,
        apiKey: body.apiKey,
        baseUrl: body.baseUrl,
        enabled: body.enabled,
      });
      res.json({ ok: true, updatedAt: saved.updatedAt });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.delete("/settings", (req: Request, res: Response) => {
    const user = (req as AuthenticatedRequest).user;
    agent.deleteSettings(user.id);
    res.json({ ok: true });
  });

  router.get("/conversations", (req: Request, res: Response) => {
    const user = (req as AuthenticatedRequest).user;
    res.json({ conversations: agent.listConversations(user.id) });
  });

  router.post("/conversations", (req: Request<unknown, unknown, ConversationBody>, res: Response) => {
    const user = (req as AuthenticatedRequest).user;
    const settings = agent.getSettings(user.id);
    if (!settings) {
      respondWithApiError(res, Errors.notFound("Agent settings"));
      return;
    }
    const conversation = agent.createConversation(user.id, settings, req.body?.title);
    res.json({ conversation });
  });

  router.get("/conversations/:id", (req: Request, res: Response) => {
    const user = (req as AuthenticatedRequest).user;
    const conversation = agent.getConversation(user.id, String(req.params.id));
    if (!conversation) {
      respondWithApiError(res, Errors.notFound("Conversation"));
      return;
    }
    const messages = agent.listMessages(conversation.id);
    res.json({ conversation, messages });
  });

  router.delete("/conversations/:id", (req: Request, res: Response) => {
    const user = (req as AuthenticatedRequest).user;
    agent.deleteConversation(user.id, String(req.params.id));
    res.json({ ok: true });
  });

  // Non-streaming send: buffers all events and returns the final messages.
  // The streaming surface is the WebSocket; this REST endpoint exists as a
  // fallback for clients that can't open a WS or want a single round-trip.
  router.post("/conversations/:id/messages", async (req: Request, res: Response) => {
    const user = (req as AuthenticatedRequest).user;
    const body: SendBody = (req.body ?? {}) as SendBody;
    const text = body.text?.trim();
    if (!text) {
      respondWithApiError(res, Errors.missingFields("text"));
      return;
    }
    const conversation = agent.getConversation(user.id, String(req.params.id));
    if (!conversation) {
      respondWithApiError(res, Errors.notFound("Conversation"));
      return;
    }

    const events: unknown[] = [];
    const ctl = new AbortController();
    try {
      await agent.send({
        user,
        conversationId: conversation.id,
        text,
        onEvent: (event) => events.push(event),
        signal: ctl.signal,
      });
      const messages = agent.listMessages(conversation.id);
      res.json({ events, messages });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/conversations/:id/cancel", (req: Request, res: Response) => {
    const user = (req as AuthenticatedRequest).user;
    const conversation = agent.getConversation(user.id, String(req.params.id));
    if (!conversation) {
      respondWithApiError(res, Errors.notFound("Conversation"));
      return;
    }
    const cancelled = agent.cancel(conversation.id);
    res.json({ ok: true, cancelled });
  });

  return router;
}

function redact(value: string): string {
  return value.length > 4 ? SECRET_PREFIX + value.slice(-4) : SECRET_PREFIX.slice(0, 4);
}
