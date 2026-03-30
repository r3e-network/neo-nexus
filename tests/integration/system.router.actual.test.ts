import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { createSystemRouter } from "../../src/api/routes/system";

describe("Actual system router", () => {
  let app: express.Application;
  let mockNodeManager: {
    cleanOldLogs: ReturnType<typeof vi.fn>;
    exportConfiguration: ReturnType<typeof vi.fn>;
    stopAllNodes: ReturnType<typeof vi.fn>;
    resetAllNodeData: ReturnType<typeof vi.fn>;
    restoreConfiguration: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());

    mockNodeManager = {
      cleanOldLogs: vi.fn(),
      exportConfiguration: vi.fn(),
      stopAllNodes: vi.fn(),
      resetAllNodeData: vi.fn(),
      restoreConfiguration: vi.fn(),
    };

    app.use("/api/system", createSystemRouter(mockNodeManager as never));
  });

  it("cleans old logs and returns the cleanup summary", async () => {
    mockNodeManager.cleanOldLogs.mockResolvedValue({
      cleanedFiles: 12,
      nodesAffected: 3,
      maxAgeDays: 30,
    });

    const response = await request(app)
      .post("/api/system/logs/clean")
      .send({ maxAgeDays: 30 });

    expect(response.status).toBe(200);
    expect(response.body).toEqual({
      cleanedFiles: 12,
      nodesAffected: 3,
      maxAgeDays: 30,
    });
  });

  it("exports node configuration as a snapshot payload", async () => {
    mockNodeManager.exportConfiguration.mockReturnValue({
      generatedAt: 1234567890,
      version: "2.0.0",
      nodes: [{ id: "node-1", name: "Mainnet Node" }],
    });

    const response = await request(app).get("/api/system/export");

    expect(response.status).toBe(200);
    expect(response.body.version).toBe("2.0.0");
    expect(response.body.nodes).toHaveLength(1);
  });

  it("stops all running nodes and reports the counts", async () => {
    mockNodeManager.stopAllNodes.mockResolvedValue({
      stoppedCount: 2,
      alreadyStoppedCount: 1,
    });

    const response = await request(app).post("/api/system/nodes/stop-all");

    expect(response.status).toBe(200);
    expect(response.body).toEqual({
      stoppedCount: 2,
      alreadyStoppedCount: 1,
    });
  });

  it("resets all node data and reports the deletion summary", async () => {
    mockNodeManager.resetAllNodeData.mockResolvedValue({
      deletedNodeCount: 4,
      removedDirectoryCount: 4,
    });

    const response = await request(app).post("/api/system/reset");

    expect(response.status).toBe(200);
    expect(response.body).toEqual({
      deletedNodeCount: 4,
      removedDirectoryCount: 4,
    });
  });

  it("returns structured error when restore payload is missing", async () => {
    const response = await request(app)
      .post("/api/system/restore")
      .send({});

    expect(response.status).toBe(400);
    expect(response.body.error).toBe("A valid snapshot payload is required");
    expect(response.body.code).toBe("SNAPSHOT_REQUIRED");
    expect(response.body.suggestion).toBeDefined();
  });

  it("restores a configuration snapshot and reports restore counts", async () => {
    mockNodeManager.restoreConfiguration.mockResolvedValue({
      restoredCount: 2,
      skippedCount: 0,
      failedCount: 1,
    });

    const response = await request(app)
      .post("/api/system/restore")
      .send({
        snapshot: {
          version: "2.0.0",
          nodes: [{ name: "Node A" }],
        },
        replaceExisting: true,
      });

    expect(response.status).toBe(200);
    expect(mockNodeManager.restoreConfiguration).toHaveBeenCalledWith(
      {
        version: "2.0.0",
        nodes: [{ name: "Node A" }],
      },
      { replaceExisting: true },
    );
    expect(response.body).toEqual({
      restoredCount: 2,
      skippedCount: 0,
      failedCount: 1,
    });
  });
});
