import { beforeEach, describe, expect, it, vi } from "vitest";
import express, { type RequestHandler } from "express";
import request from "supertest";
import { createAgentRouter } from "../../src/api/routes/agent";
import type { AuthenticatedRequest } from "../../src/api/middleware/auth";

describe("Actual agent router", () => {
  let app: express.Application;
  let agent: {
    isEnabled: ReturnType<typeof vi.fn>;
    getSettings: ReturnType<typeof vi.fn>;
    saveSettings: ReturnType<typeof vi.fn>;
    deleteSettings: ReturnType<typeof vi.fn>;
    listConversations: ReturnType<typeof vi.fn>;
    createConversation: ReturnType<typeof vi.fn>;
    getConversation: ReturnType<typeof vi.fn>;
    listMessages: ReturnType<typeof vi.fn>;
    deleteConversation: ReturnType<typeof vi.fn>;
    send: ReturnType<typeof vi.fn>;
    cancel: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());

    const auth: RequestHandler = (req, _res, next) => {
      (req as AuthenticatedRequest).user = { id: "user-1", username: "alice", role: "admin" };
      next();
    };

    agent = {
      isEnabled: vi.fn(() => true),
      getSettings: vi.fn(() => null),
      saveSettings: vi.fn(() => ({
        userId: "user-1",
        provider: "anthropic",
        model: "claude-sonnet-4-6",
        apiKey: "sk-ant-real-secret",
        enabled: true,
        createdAt: 1,
        updatedAt: 2,
      })),
      deleteSettings: vi.fn(),
      listConversations: vi.fn(() => []),
      createConversation: vi.fn(),
      getConversation: vi.fn(),
      listMessages: vi.fn(() => []),
      deleteConversation: vi.fn(),
      send: vi.fn(),
      cancel: vi.fn(),
    };

    app.use("/api/agent", auth, createAgentRouter(agent as never));
  });

  it("redacts API keys in settings responses", async () => {
    agent.getSettings.mockReturnValue({
      userId: "user-1",
      provider: "anthropic",
      model: "claude-sonnet-4-6",
      apiKey: "sk-ant-real-secret",
      enabled: true,
      createdAt: 1,
      updatedAt: 2,
    });

    const response = await request(app).get("/api/agent/settings");

    expect(response.status).toBe(200);
    expect(response.body.apiKey).toBe("••••••••...cret");
    expect(JSON.stringify(response.body)).not.toContain("sk-ant-real-secret");
  });

  it("preserves the existing API key when settings are saved with the current redacted placeholder", async () => {
    agent.getSettings.mockReturnValue({
      userId: "user-1",
      provider: "anthropic",
      model: "claude-sonnet-4-6",
      apiKey: "sk-ant-real-secret",
      enabled: true,
      createdAt: 1,
      updatedAt: 2,
    });

    const response = await request(app)
      .put("/api/agent/settings")
      .send({
        provider: "anthropic",
        model: "claude-opus-4-1",
        apiKey: "••••••••...cret",
        enabled: true,
      });

    expect(response.status).toBe(200);
    expect(agent.saveSettings).toHaveBeenCalledWith("user-1", {
      provider: "anthropic",
      model: "claude-opus-4-1",
      apiKey: "sk-ant-real-secret",
      baseUrl: undefined,
      enabled: true,
    });
  });

  it("rejects redacted API key placeholders that do not match the stored secret", async () => {
    agent.getSettings.mockReturnValue({
      userId: "user-1",
      provider: "anthropic",
      model: "claude-sonnet-4-6",
      apiKey: "sk-ant-real-secret",
      enabled: true,
      createdAt: 1,
      updatedAt: 2,
    });

    const response = await request(app)
      .put("/api/agent/settings")
      .send({
        provider: "anthropic",
        model: "claude-opus-4-1",
        apiKey: "••••••••...fake",
        enabled: true,
      });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("AGENT_API_KEY_PLACEHOLDER_INVALID");
    expect(agent.saveSettings).not.toHaveBeenCalled();
  });
});
