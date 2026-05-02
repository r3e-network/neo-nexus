import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import type { RequestHandler } from "express";
import request from "supertest";
import { createPluginsRouter } from "../../src/api/routes/plugins";
import { Errors } from "../../src/api/errors";
import type { AuthenticatedRequest } from "../../src/api/middleware/auth";

describe("Actual plugins router", () => {
  let app: express.Application;
  let nodeManager: {
    getPluginManager: ReturnType<typeof vi.fn>;
    installPlugin: ReturnType<typeof vi.fn>;
    updatePluginConfig: ReturnType<typeof vi.fn>;
    uninstallPlugin: ReturnType<typeof vi.fn>;
    setPluginEnabled: ReturnType<typeof vi.fn>;
  };
  let pluginManager: {
    getInstalledPlugins: ReturnType<typeof vi.fn>;
    getAvailablePlugins: ReturnType<typeof vi.fn>;
  };
  const viewerMiddleware: RequestHandler = (req, _res, next) => {
    (req as AuthenticatedRequest).user = { id: "viewer-1", username: "viewer", role: "viewer" };
    next();
  };

  const createViewerApp = () => {
    const viewerApp = express();
    viewerApp.use(express.json());
    viewerApp.use(viewerMiddleware);
    viewerApp.use("/api/nodes/:id/plugins", createPluginsRouter(nodeManager as never));
    return viewerApp;
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());
    pluginManager = {
      getInstalledPlugins: vi.fn(() => [{ id: "RpcServer", enabled: true }]),
      getAvailablePlugins: vi.fn(() => [{ id: "RpcServer", name: "RPC Server" }]),
    };
    nodeManager = {
      getPluginManager: vi.fn(() => pluginManager),
      installPlugin: vi.fn().mockResolvedValue(undefined),
      updatePluginConfig: vi.fn(),
      uninstallPlugin: vi.fn().mockResolvedValue(undefined),
      setPluginEnabled: vi.fn().mockResolvedValue(undefined),
    };
    app.use("/api/nodes/:id/plugins", createPluginsRouter(nodeManager as never));
  });

  it("requires a pluginId when installing", async () => {
    const response = await request(app).post("/api/nodes/node-1/plugins").send({});

    expect(response.status).toBe(400);
    expect(response.body.error).toMatch(/pluginId/i);
    expect(nodeManager.installPlugin).not.toHaveBeenCalled();
  });

  it("installs a plugin and returns the refreshed installed list", async () => {
    const response = await request(app)
      .post("/api/nodes/node-1/plugins")
      .send({ pluginId: "RpcServer", config: { Port: 10332 } });

    expect(response.status).toBe(201);
    expect(nodeManager.installPlugin).toHaveBeenCalledWith("node-1", "RpcServer", { Port: 10332 });
    expect(response.body.plugins).toEqual([{ id: "RpcServer", enabled: true }]);
  });

  it("redacts installed plugin configuration for viewer reads", async () => {
    const viewerApp = createViewerApp();
    pluginManager.getInstalledPlugins.mockReturnValue([
      {
        id: "RpcServer",
        version: "3.9.2",
        config: { Port: 10332, Password: "plugin-secret" },
        installedAt: 123,
        enabled: true,
      },
    ]);

    const response = await request(viewerApp).get("/api/nodes/node-1/plugins");

    expect(response.status).toBe(200);
    expect(response.body.plugins).toEqual([
      {
        id: "RpcServer",
        version: "3.9.2",
        installedAt: 123,
        enabled: true,
      },
    ]);
    expect(JSON.stringify(response.body)).not.toContain("plugin-secret");
  });

  it("lists available plugins from the catalog", async () => {
    pluginManager.getAvailablePlugins.mockReturnValue([
      { id: "RpcServer", name: "RPC Server", category: "API" },
    ]);

    const response = await request(app).get("/api/nodes/node-1/plugins/available");

    expect(response.status).toBe(200);
    expect(pluginManager.getAvailablePlugins).toHaveBeenCalled();
    expect(response.body.plugins).toEqual([{ id: "RpcServer", name: "RPC Server", category: "API" }]);
  });

  it("updates plugin config and returns the refreshed installed list", async () => {
    const response = await request(app)
      .put("/api/nodes/node-1/plugins/RpcServer")
      .send({ config: { Port: 20332 } });

    expect(response.status).toBe(200);
    expect(nodeManager.updatePluginConfig).toHaveBeenCalledWith("node-1", "RpcServer", { Port: 20332 });
    expect(response.body.plugins).toEqual([{ id: "RpcServer", enabled: true }]);
  });

  it("uninstalls a plugin with the real 204 contract", async () => {
    const response = await request(app).delete("/api/nodes/node-1/plugins/RpcServer");

    expect(response.status).toBe(204);
    expect(nodeManager.uninstallPlugin).toHaveBeenCalledWith("node-1", "RpcServer");
  });

  it("disables a plugin and returns the refreshed installed list", async () => {
    const response = await request(app).post("/api/nodes/node-1/plugins/RpcServer/disable");

    expect(response.status).toBe(200);
    expect(nodeManager.setPluginEnabled).toHaveBeenCalledWith("node-1", "RpcServer", false);
    expect(response.body.plugins).toEqual([{ id: "RpcServer", enabled: true }]);
  });

  it("maps missing plugin mutations to 404", async () => {
    nodeManager.setPluginEnabled.mockRejectedValue(Errors.pluginNotInstalled("RpcServer", "node-1"));

    const response = await request(app).post("/api/nodes/node-1/plugins/RpcServer/enable");

    expect(response.status).toBe(404);
    expect(response.body.error).toMatch(/not installed/i);
  });

  it("maps running-node plugin mutations to 409", async () => {
    nodeManager.installPlugin.mockRejectedValue(Errors.nodeRunning());

    const response = await request(app)
      .post("/api/nodes/node-1/plugins")
      .send({ pluginId: "RpcServer" });

    expect(response.status).toBe(409);
    expect(response.body.error).toMatch(/running/i);
  });
});
