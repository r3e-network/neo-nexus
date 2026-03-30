import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { createServersRouter } from "../../src/api/routes/servers";

describe("Actual servers router", () => {
  let app: express.Application;
  let mockManager: {
    listServersWithStatus: ReturnType<typeof vi.fn>;
    getServerSummary: ReturnType<typeof vi.fn>;
    createServer: ReturnType<typeof vi.fn>;
    updateServer: ReturnType<typeof vi.fn>;
    deleteServer: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());

    mockManager = {
      listServersWithStatus: vi.fn(),
      getServerSummary: vi.fn(),
      createServer: vi.fn(),
      updateServer: vi.fn(),
      deleteServer: vi.fn(),
    };

    app.use("/api/servers", createServersRouter(mockManager as never));
  });

  it("lists server profiles with live status", async () => {
    mockManager.listServersWithStatus.mockResolvedValue([
      { profile: { id: "srv-1", name: "Tokyo", baseUrl: "https://tokyo.example.com" }, reachable: true },
    ]);

    const response = await request(app).get("/api/servers");

    expect(response.status).toBe(200);
    expect(response.body.servers).toHaveLength(1);
  });

  it("returns a single server summary", async () => {
    mockManager.getServerSummary.mockResolvedValue({
      profile: { id: "srv-1", name: "Tokyo", baseUrl: "https://tokyo.example.com" },
      reachable: true,
      status: { totalNodes: 2 },
    });

    const response = await request(app).get("/api/servers/srv-1");

    expect(response.status).toBe(200);
    expect(response.body.server.profile.id).toBe("srv-1");
  });

  it("creates a server profile", async () => {
    mockManager.createServer.mockReturnValue({
      id: "srv-1",
      name: "Tokyo",
      baseUrl: "https://tokyo.example.com",
      enabled: true,
    });

    const response = await request(app)
      .post("/api/servers")
      .send({
        name: "Tokyo",
        baseUrl: "https://tokyo.example.com",
      });

    expect(response.status).toBe(201);
    expect(response.body.server.id).toBe("srv-1");
  });

  it("updates a server profile", async () => {
    mockManager.updateServer.mockReturnValue({
      id: "srv-1",
      name: "Tokyo Updated",
      baseUrl: "https://tokyo.example.com",
      enabled: true,
    });

    const response = await request(app)
      .put("/api/servers/srv-1")
      .send({
        name: "Tokyo Updated",
      });

    expect(response.status).toBe(200);
    expect(response.body.server.name).toBe("Tokyo Updated");
  });

  it("returns structured error when required fields are missing on create", async () => {
    const response = await request(app)
      .post("/api/servers")
      .send({ name: "Incomplete" });

    expect(response.status).toBe(400);
    expect(response.body.error).toBe("Missing required fields: name, baseUrl");
    expect(response.body.code).toBe("MISSING_FIELDS");
    expect(response.body.suggestion).toBeDefined();
  });

  it("deletes a server profile", async () => {
    mockManager.deleteServer.mockReturnValue(undefined);

    const response = await request(app).delete("/api/servers/srv-1");

    expect(response.status).toBe(204);
  });
});
