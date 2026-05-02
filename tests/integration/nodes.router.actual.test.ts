import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import type { RequestHandler } from "express";
import request from "supertest";
import { createNodesRouter } from "../../src/api/routes/nodes";
import { Errors } from "../../src/api/errors";
import type { AuthenticatedRequest } from "../../src/api/middleware/auth";
import { ConfigManager } from "../../src/core/ConfigManager";

type MockNodeManager = {
  createNode: ReturnType<typeof vi.fn>;
  importExistingNode: ReturnType<typeof vi.fn>;
  getNode: ReturnType<typeof vi.fn>;
  getAllNodes: ReturnType<typeof vi.fn>;
  updateNode: ReturnType<typeof vi.fn>;
  updateImportedNodeOwnership: ReturnType<typeof vi.fn>;
  deleteNode: ReturnType<typeof vi.fn>;
  startNode: ReturnType<typeof vi.fn>;
  stopNode: ReturnType<typeof vi.fn>;
  restartNode: ReturnType<typeof vi.fn>;
  getNodeLogs: ReturnType<typeof vi.fn>;
  getStorageInfo: ReturnType<typeof vi.fn>;
  getNodeSecureSignerHealth: ReturnType<typeof vi.fn>;
  syncNodeSecureSigner: ReturnType<typeof vi.fn>;
  getSecureSignerManager: ReturnType<typeof vi.fn>;
  getPluginManager: ReturnType<typeof vi.fn>;
};

describe("Actual nodes router", () => {
  let app: express.Application;
  let mockNodeManager: MockNodeManager;
  let getProfile: ReturnType<typeof vi.fn>;
  const viewerMiddleware: RequestHandler = (req, _res, next) => {
    (req as AuthenticatedRequest).user = { id: "viewer-1", username: "viewer", role: "viewer" };
    next();
  };

  const createViewerApp = () => {
    const viewerApp = express();
    viewerApp.use(express.json());
    viewerApp.use(viewerMiddleware);
    viewerApp.use("/api/nodes", createNodesRouter(mockNodeManager as never));
    return viewerApp;
  };

  beforeEach(() => {
    vi.restoreAllMocks();
    app = express();
    app.use(express.json());

    getProfile = vi.fn();

    mockNodeManager = {
      createNode: vi.fn(),
      importExistingNode: vi.fn(),
      getNode: vi.fn(),
      getAllNodes: vi.fn(() => []),
      updateNode: vi.fn(),
      updateImportedNodeOwnership: vi.fn(),
      deleteNode: vi.fn(),
      startNode: vi.fn(),
      stopNode: vi.fn(),
      restartNode: vi.fn(),
      getNodeLogs: vi.fn(() => []),
      getStorageInfo: vi.fn(),
      getNodeSecureSignerHealth: vi.fn(),
      syncNodeSecureSigner: vi.fn(),
      getSecureSignerManager: vi.fn(() => ({
        getProfile,
      })),
      getPluginManager: vi.fn(() => ({
        getInstalledPlugins: vi.fn(() => []),
      })),
    };

    app.use("/api/nodes", createNodesRouter(mockNodeManager as never));
  });

  it("lists all nodes from the real router", async () => {
    mockNodeManager.getAllNodes.mockReturnValue([
      { id: "node-1", name: "Node 1", type: "neo-cli" },
      { id: "node-2", name: "Node 2", type: "neo-go" },
    ]);

    const response = await request(app).get("/api/nodes");

    expect(response.status).toBe(200);
    expect(response.body).toEqual({
      nodes: [
        { id: "node-1", name: "Node 1", type: "neo-cli" },
        { id: "node-2", name: "Node 2", type: "neo-go" },
      ],
    });
  });

  it("redacts path, custom config, and plugin config data for viewer node lists", async () => {
    const viewerApp = createViewerApp();
    mockNodeManager.getAllNodes.mockReturnValue([
      {
        id: "node-1",
        name: "Node 1",
        type: "neo-cli",
        network: "mainnet",
        syncMode: "full",
        version: "3.9.2",
        ports: { rpc: 10332, p2p: 10333 },
        paths: {
          base: "/home/operator/.neonexus/nodes/node-1",
          data: "/home/operator/.neonexus/nodes/node-1/data",
          logs: "/home/operator/.neonexus/nodes/node-1/logs",
          config: "/home/operator/.neonexus/nodes/node-1/config.json",
          wallet: "/home/operator/.neonexus/nodes/node-1/wallet.json",
        },
        settings: {
          customConfig: { RpcPassword: "raw-secret" },
          keyProtection: { mode: "secure-signer", signerProfileId: "signer-secret" },
          import: { imported: true, ownershipMode: "managed-config", existingPath: "/opt/secret-node", importedAt: 123 },
        },
        process: { status: "running", pid: 1234 },
        metrics: { blockHeight: 1, headerHeight: 1, connectedPeers: 2, unconnectedPeers: 0, syncProgress: 100, memoryUsage: 10, cpuUsage: 1, lastUpdate: 456 },
        plugins: [{ id: "RpcServer", version: "3.9.2", config: { Password: "plugin-secret" }, installedAt: 789, enabled: true }],
        createdAt: 1,
        updatedAt: 2,
      },
    ]);

    const response = await request(viewerApp).get("/api/nodes");

    expect(response.status).toBe(200);
    const node = response.body.nodes[0];
    expect(node.paths).toBeUndefined();
    expect(node.settings.customConfig).toBeUndefined();
    expect(node.settings.keyProtection).toEqual({ mode: "secure-signer" });
    expect(node.settings.import).toEqual({ imported: true, ownershipMode: "managed-config" });
    expect(node.plugins[0]).toEqual({ id: "RpcServer", version: "3.9.2", installedAt: 789, enabled: true });
    expect(JSON.stringify(response.body)).not.toContain("raw-secret");
    expect(JSON.stringify(response.body)).not.toContain("plugin-secret");
    expect(JSON.stringify(response.body)).not.toContain("/home/operator");
    expect(JSON.stringify(response.body)).not.toContain("/opt/secret-node");
    expect(JSON.stringify(response.body)).not.toContain("signer-secret");
  });

  it("redacts path and custom config data for viewer node details", async () => {
    const viewerApp = createViewerApp();
    mockNodeManager.getNode.mockReturnValue({
      id: "node-1",
      name: "Node 1",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
      version: "3.9.2",
      ports: { rpc: 10332, p2p: 10333 },
      paths: { base: "/home/operator/node-1", data: "/home/operator/node-1/data", logs: "/home/operator/node-1/logs", config: "/home/operator/node-1/config.json" },
      settings: { customConfig: { Password: "raw-secret" } },
      process: { status: "stopped" },
      plugins: [{ id: "RpcServer", version: "3.9.2", config: { Password: "plugin-secret" }, installedAt: 789, enabled: true }],
      createdAt: 1,
      updatedAt: 2,
    });

    const response = await request(viewerApp).get("/api/nodes/node-1");

    expect(response.status).toBe(200);
    expect(response.body.node.paths).toBeUndefined();
    expect(response.body.node.settings.customConfig).toBeUndefined();
    expect(response.body.node.plugins[0].config).toBeUndefined();
    expect(JSON.stringify(response.body)).not.toContain("raw-secret");
    expect(JSON.stringify(response.body)).not.toContain("plugin-secret");
    expect(JSON.stringify(response.body)).not.toContain("/home/operator");
  });

  it("blocks config audit for viewer users", async () => {
    const viewerApp = createViewerApp();

    const response = await request(viewerApp).get("/api/nodes/node-1/config-audit");

    expect(response.status).toBe(403);
    expect(response.body.code).toBe("ADMIN_REQUIRED");
    expect(mockNodeManager.getNode).not.toHaveBeenCalled();
  });

  it("audits admin config using the current node and installed plugin IDs", async () => {
    const node = {
      id: "node-1",
      name: "Node 1",
      type: "neo-cli",
      network: "mainnet",
      version: "3.9.2",
      paths: { config: "/tmp/config.json" },
      settings: {},
    };
    const getInstalledPlugins = vi.fn(() => [
      { id: "RpcServer" },
      { id: "ApplicationLogs" },
    ]);
    const audit = {
      nodeId: "node-1",
      nodeName: "Node 1",
      nodeType: "neo-cli",
      version: "3.9.2",
      network: "mainnet",
      issueCount: 0,
      errors: 0,
      warnings: 0,
      info: 0,
      issues: [],
    };
    const auditSpy = vi.spyOn(ConfigManager, "auditNodeConfig").mockResolvedValueOnce(audit);
    mockNodeManager.getNode.mockReturnValue(node);
    mockNodeManager.getPluginManager.mockReturnValue({ getInstalledPlugins });

    const response = await request(app).get("/api/nodes/node-1/config-audit");

    expect(response.status).toBe(200);
    expect(getInstalledPlugins).toHaveBeenCalledWith("node-1");
    expect(auditSpy).toHaveBeenCalledWith(node, ["RpcServer", "ApplicationLogs"]);
    expect(response.body.audit).toEqual(audit);
  });

  it("creates a node with valid input", async () => {
    mockNodeManager.createNode.mockResolvedValue({
      id: "node-new",
      name: "New Node",
      type: "neo-cli",
      network: "mainnet",
    });

    const response = await request(app).post("/api/nodes").send({
      name: "New Node",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
    });

    expect(response.status).toBe(201);
    expect(mockNodeManager.createNode).toHaveBeenCalledWith({
      name: "New Node",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
    });
    expect(response.body.node.id).toBe("node-new");
  });

  it("rejects creation requests with missing required fields before touching the manager", async () => {
    const response = await request(app).post("/api/nodes").send({
      type: "neo-cli",
    });

    expect(response.status).toBe(400);
    expect(response.body.error).toMatch(/missing required fields/i);
    expect(response.body.code).toBe("MISSING_FIELDS");
    expect(response.body.suggestion).toBeTruthy();
    expect(mockNodeManager.createNode).not.toHaveBeenCalled();
  });

  it("rejects creation requests with invalid node type before touching the manager", async () => {
    const response = await request(app).post("/api/nodes").send({
      name: "Invalid Type",
      type: "neo-js",
      network: "mainnet",
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("INVALID_NODE_TYPE");
    expect(mockNodeManager.createNode).not.toHaveBeenCalled();
  });

  it("rejects creation requests with invalid network before touching the manager", async () => {
    const response = await request(app).post("/api/nodes").send({
      name: "Invalid Network",
      type: "neo-cli",
      network: "staging",
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("INVALID_NODE_NETWORK");
    expect(mockNodeManager.createNode).not.toHaveBeenCalled();
  });

  it("imports an existing node through the real router", async () => {
    mockNodeManager.importExistingNode.mockResolvedValue({
      id: "node-imported",
      name: "Imported Node",
    });

    const response = await request(app).post("/api/nodes/import").send({
      name: "Imported Node",
      existingPath: "/home/neo/imported-node",
    });

    expect(response.status).toBe(201);
    expect(mockNodeManager.importExistingNode).toHaveBeenCalledWith({
      name: "Imported Node",
      existingPath: "/home/neo/imported-node",
    });
    expect(response.body.node.id).toBe("node-imported");
  });

  it("rejects import paths that only share an allowed prefix", async () => {
    mockNodeManager.importExistingNode.mockResolvedValue({ id: "should-not-import" });

    const response = await request(app).post("/api/nodes/import").send({
      name: "Bad Import",
      existingPath: "/home2/neo-node",
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("PATH_NOT_ALLOWED");
    expect(mockNodeManager.importExistingNode).not.toHaveBeenCalled();
  });

  it("forwards valid imported-node ownership modes", async () => {
    mockNodeManager.importExistingNode.mockResolvedValue({ id: "node-imported" });

    const response = await request(app).post("/api/nodes/import").send({
      name: "Imported Node",
      existingPath: "/home/neo/imported-node",
      ownershipMode: "managed-process",
    });

    expect(response.status).toBe(201);
    expect(mockNodeManager.importExistingNode).toHaveBeenCalledWith(expect.objectContaining({
      ownershipMode: "managed-process",
    }));
  });

  it("rejects invalid imported-node ownership modes", async () => {
    const response = await request(app).post("/api/nodes/import").send({
      name: "Imported Node",
      existingPath: "/home/neo/imported-node",
      ownershipMode: "root-everything",
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("INVALID_OWNERSHIP_MODE");
    expect(mockNodeManager.importExistingNode).not.toHaveBeenCalled();
  });

  it("updates imported-node ownership through the dedicated endpoint", async () => {
    mockNodeManager.updateImportedNodeOwnership.mockResolvedValue({
      id: "node-imported",
      settings: { import: { imported: true, ownershipMode: "managed-config" } },
    });

    const response = await request(app).post("/api/nodes/node-imported/ownership").send({
      ownershipMode: "managed-config",
    });

    expect(response.status).toBe(200);
    expect(mockNodeManager.updateImportedNodeOwnership).toHaveBeenCalledWith("node-imported", "managed-config");
    expect(response.body.node.settings.import.ownershipMode).toBe("managed-config");
  });

  it("rejects ownership endpoint requests without an ownership mode", async () => {
    const response = await request(app).post("/api/nodes/node-imported/ownership").send({});

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("MISSING_FIELDS");
    expect(mockNodeManager.updateImportedNodeOwnership).not.toHaveBeenCalled();
  });

  it("rejects ownership endpoint requests with invalid ownership modes", async () => {
    const response = await request(app).post("/api/nodes/node-imported/ownership").send({
      ownershipMode: "root-everything",
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("INVALID_OWNERSHIP_MODE");
    expect(mockNodeManager.updateImportedNodeOwnership).not.toHaveBeenCalled();
  });

  it("maps non-imported ownership updates to a 400 permission error", async () => {
    mockNodeManager.updateImportedNodeOwnership.mockRejectedValue(Errors.nodeOwnershipNotImported());

    const response = await request(app).post("/api/nodes/managed-node/ownership").send({
      ownershipMode: "managed-config",
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("NODE_OWNERSHIP_NOT_IMPORTED");
  });

  it("returns node details when the node exists", async () => {
    mockNodeManager.getNode.mockReturnValue({
      id: "node-1",
      name: "Test Node",
      type: "neo-cli",
    });

    const response = await request(app).get("/api/nodes/node-1");

    expect(response.status).toBe(200);
    expect(response.body.node.id).toBe("node-1");
  });

  it("returns 404 when a node does not exist", async () => {
    mockNodeManager.getNode.mockReturnValue(null);

    const response = await request(app).get("/api/nodes/missing-node");

    expect(response.status).toBe(404);
    expect(response.body.error).toMatch(/not found/i);
    expect(response.body.code).toBe("NODE_NOT_FOUND");
    expect(response.body.suggestion).toBeTruthy();
  });

  it("rejects secure signer creation without a signer profile", async () => {
    const response = await request(app).post("/api/nodes").send({
      name: "Protected Node",
      type: "neo-cli",
      network: "mainnet",
      settings: {
        keyProtection: {
          mode: "secure-signer",
        },
      },
    });

    expect(response.status).toBe(400);
    expect(response.body.error).toMatch(/signer profile/i);
    expect(response.body.code).toBe("SIGNER_REQUIRES_PROFILE");
    expect(mockNodeManager.createNode).not.toHaveBeenCalled();
  });

  it("rejects secure signer creation for neo-go nodes", async () => {
    const response = await request(app).post("/api/nodes").send({
      name: "Protected Neo Go",
      type: "neo-go",
      network: "mainnet",
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    });

    expect(response.status).toBe(400);
    expect(response.body.error).toMatch(/neo-cli/i);
    expect(response.body.code).toBe("SIGNER_NEO_CLI_ONLY");
    expect(mockNodeManager.createNode).not.toHaveBeenCalled();
  });

  it("rejects secure signer creation when the referenced profile is disabled or missing", async () => {
    getProfile.mockReturnValue(null);

    const response = await request(app).post("/api/nodes").send({
      name: "Protected Node",
      type: "neo-cli",
      network: "mainnet",
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    });

    expect(response.status).toBe(400);
    expect(response.body.error).toMatch(/not available/i);
    expect(response.body.code).toBe("SIGNER_NOT_AVAILABLE");
    expect(mockNodeManager.createNode).not.toHaveBeenCalled();
  });

  it("updates a node and delegates secure signer synchronization to NodeManager", async () => {
    getProfile.mockReturnValue({ id: "signer-1", enabled: true });
    mockNodeManager.getNode.mockReturnValue({ id: "node-1", type: "neo-cli" });
    mockNodeManager.updateNode.mockResolvedValue({
      id: "node-1",
      name: "Updated Node",
    });

    const response = await request(app).put("/api/nodes/node-1").send({
      name: "Updated Node",
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    });

    expect(response.status).toBe(200);
    expect(mockNodeManager.updateNode).toHaveBeenCalledWith("node-1", {
      name: "Updated Node",
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    });
    expect(mockNodeManager.syncNodeSecureSigner).not.toHaveBeenCalled();
    expect(response.body.node.name).toBe("Updated Node");
  });

  it("rejects secure signer updates for existing neo-go nodes before mutating", async () => {
    mockNodeManager.getNode.mockReturnValue({ id: "node-1", type: "neo-go" });

    const response = await request(app).put("/api/nodes/node-1").send({
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("SIGNER_NEO_CLI_ONLY");
    expect(mockNodeManager.updateNode).not.toHaveBeenCalled();
  });

  it("rejects secure signer updates when the referenced profile is unavailable", async () => {
    mockNodeManager.getNode.mockReturnValue({ id: "node-1", type: "neo-cli" });
    getProfile.mockReturnValue({ id: "signer-1", enabled: false });

    const response = await request(app).put("/api/nodes/node-1").send({
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("SIGNER_NOT_AVAILABLE");
    expect(mockNodeManager.updateNode).not.toHaveBeenCalled();
  });

  it("returns 404 for update requests when the node does not exist", async () => {
    mockNodeManager.getNode.mockReturnValue(null);

    const response = await request(app).put("/api/nodes/missing-node").send({
      name: "Updated Node",
    });

    expect(response.status).toBe(404);
    expect(response.body.code).toBe("NODE_NOT_FOUND");
    expect(mockNodeManager.updateNode).not.toHaveBeenCalled();
  });

  it("deletes a node with the real 204 contract", async () => {
    mockNodeManager.deleteNode.mockResolvedValue(undefined);

    const response = await request(app).delete("/api/nodes/node-1");

    expect(response.status).toBe(204);
    expect(response.text).toBe("");
  });

  it("starts a node and returns the refreshed node payload", async () => {
    mockNodeManager.startNode.mockResolvedValue(undefined);
    mockNodeManager.getNode.mockReturnValue({ id: "node-1", process: { status: "running" } });

    const response = await request(app).post("/api/nodes/node-1/start");

    expect(response.status).toBe(200);
    expect(mockNodeManager.startNode).toHaveBeenCalledWith("node-1");
    expect(response.body.node.process.status).toBe("running");
  });

  it("maps not-found start failures to 404", async () => {
    mockNodeManager.startNode.mockRejectedValue(new Error("Node not found"));

    const response = await request(app).post("/api/nodes/missing-node/start");

    expect(response.status).toBe(404);
    expect(response.body.error).toMatch(/not found/i);
    expect(response.body.code).toBeTruthy();
  });

  it("stops a node and forwards the force flag", async () => {
    mockNodeManager.stopNode.mockResolvedValue(undefined);
    mockNodeManager.getNode.mockReturnValue({ id: "node-1", process: { status: "stopped" } });

    const response = await request(app).post("/api/nodes/node-1/stop").send({ force: true });

    expect(response.status).toBe(200);
    expect(mockNodeManager.stopNode).toHaveBeenCalledWith("node-1", true);
    expect(response.body.node.process.status).toBe("stopped");
  });

  it("restarts a node and returns the refreshed node payload", async () => {
    mockNodeManager.restartNode.mockResolvedValue(undefined);
    mockNodeManager.getNode.mockReturnValue({ id: "node-1", process: { status: "running" } });

    const response = await request(app).post("/api/nodes/node-1/restart");

    expect(response.status).toBe(200);
    expect(mockNodeManager.restartNode).toHaveBeenCalledWith("node-1");
    expect(response.body.node.id).toBe("node-1");
  });

  it("returns logs for an existing node and clamps count to the router limit", async () => {
    mockNodeManager.getNode.mockReturnValue({
      id: "node-1",
      paths: { logs: "/tmp/node-1/logs" },
    });
    mockNodeManager.getNodeLogs.mockReturnValue(["Recent log"]);

    const response = await request(app).get("/api/nodes/node-1/logs?count=5000");

    expect(response.status).toBe(200);
    expect(mockNodeManager.getNodeLogs).toHaveBeenCalledWith("node-1", 1000);
    expect(response.body.logs).toEqual(["Recent log"]);
  });

  it("redacts storage paths for viewer storage reads", async () => {
    const viewerApp = createViewerApp();
    mockNodeManager.getStorageInfo.mockResolvedValue({
      chain: { size: 1024, path: "/secret/node/data" },
      logs: { size: 256, files: 3 },
      wallets: { count: 1, path: "/secret/node/wallets" },
    });

    const response = await request(viewerApp).get("/api/nodes/node-1/storage");

    expect(response.status).toBe(200);
    expect(response.body.storage).toEqual({
      chain: { size: 1024 },
      logs: { size: 256, files: 3 },
      wallets: { count: 1 },
    });
    expect(JSON.stringify(response.body)).not.toContain("/secret/node");
  });

  it("returns null signer health for standard-wallet nodes", async () => {
    mockNodeManager.getNode.mockReturnValue({ id: "node-1" });
    mockNodeManager.getNodeSecureSignerHealth.mockResolvedValue(null);

    const response = await request(app).get("/api/nodes/node-1/signer-health");

    expect(response.status).toBe(200);
    expect(response.body.signerHealth).toBeNull();
  });

  it("returns bound signer health for protected nodes", async () => {
    mockNodeManager.getNode.mockReturnValue({ id: "node-1" });
    mockNodeManager.getNodeSecureSignerHealth.mockResolvedValue({
      nodeId: "node-1",
      profile: {
        id: "signer-1",
        name: "Nitro Council",
        mode: "nitro",
        endpoint: "vsock://2345:9991",
      },
      readiness: {
        ok: true,
        status: "reachable",
        source: "secure-sign-tools",
        accountStatus: "Single",
        message: "Signer status checked.",
        checkedAt: Date.now(),
      },
    });

    const response = await request(app).get("/api/nodes/node-1/signer-health");

    expect(response.status).toBe(200);
    expect(response.body.signerHealth.profile.id).toBe("signer-1");
    expect(response.body.signerHealth.readiness.accountStatus).toBe("Single");
  });

  it("redacts signer profile identifiers and endpoints for viewer signer-health reads", async () => {
    const viewerApp = createViewerApp();
    mockNodeManager.getNode.mockReturnValue({ id: "node-1" });
    mockNodeManager.getNodeSecureSignerHealth.mockResolvedValue({
      nodeId: "node-1",
      profile: {
        id: "signer-secret",
        name: "Nitro Council",
        mode: "nitro",
        endpoint: "vsock://2345:9991",
      },
      readiness: {
        ok: true,
        status: "reachable",
        source: "secure-sign-tools",
        accountStatus: "Single",
        message: "Signer status checked.",
        checkedAt: 123,
      },
    });

    const response = await request(viewerApp).get("/api/nodes/node-1/signer-health");

    expect(response.status).toBe(200);
    expect(response.body.signerHealth).toEqual({
      nodeId: "node-1",
      profile: {
        name: "Nitro Council",
        mode: "nitro",
      },
      readiness: {
        ok: true,
        status: "reachable",
        source: "secure-sign-tools",
        accountStatus: "Single",
        message: "Signer status checked.",
        checkedAt: 123,
      },
    });
    expect(JSON.stringify(response.body)).not.toContain("signer-secret");
    expect(JSON.stringify(response.body)).not.toContain("vsock://2345:9991");
  });
});
