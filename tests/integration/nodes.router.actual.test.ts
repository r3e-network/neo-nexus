import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { createNodesRouter } from "../../src/api/routes/nodes";

type MockNodeManager = {
  createNode: ReturnType<typeof vi.fn>;
  importExistingNode: ReturnType<typeof vi.fn>;
  getNode: ReturnType<typeof vi.fn>;
  getAllNodes: ReturnType<typeof vi.fn>;
  updateNode: ReturnType<typeof vi.fn>;
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

  beforeEach(() => {
    app = express();
    app.use(express.json());

    getProfile = vi.fn();

    mockNodeManager = {
      createNode: vi.fn(),
      importExistingNode: vi.fn(),
      getNode: vi.fn(),
      getAllNodes: vi.fn(() => []),
      updateNode: vi.fn(),
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

  it("updates a node and syncs the secure signer when requested", async () => {
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
    expect(mockNodeManager.syncNodeSecureSigner).toHaveBeenCalledWith("node-1");
    expect(response.body.node.name).toBe("Updated Node");
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
});
