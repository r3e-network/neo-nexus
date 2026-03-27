import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { createNodesRouter } from "../../src/api/routes/nodes";

describe("Actual nodes router", () => {
  let app: express.Application;
  let mockNodeManager: {
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
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());

    const getProfile = vi.fn();

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
    };

    app.use("/api/nodes", createNodesRouter(mockNodeManager as never));
  });

  it("rejects secure signer creation without a signer profile", async () => {
    const response = await request(app)
      .post("/api/nodes")
      .send({
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
    expect(mockNodeManager.createNode).not.toHaveBeenCalled();
  });

  it("rejects secure signer creation for neo-go nodes", async () => {
    const response = await request(app)
      .post("/api/nodes")
      .send({
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
    expect(mockNodeManager.createNode).not.toHaveBeenCalled();
  });

  it("rejects secure signer creation when the referenced profile is disabled or missing", async () => {
    mockNodeManager.getSecureSignerManager().getProfile.mockReturnValue(null);

    const response = await request(app)
      .post("/api/nodes")
      .send({
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
    expect(mockNodeManager.createNode).not.toHaveBeenCalled();
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
