import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { ApiError } from "../../src/api/errors";
import { createPrivateNetworksRouter } from "../../src/api/routes/privateNetworks";
import type { ConfigurationSnapshot, PrivateNetworkPlan } from "../../src/types";

function createPlan(overrides: Partial<PrivateNetworkPlan> = {}): PrivateNetworkPlan {
  return {
    id: "plan-1",
    name: "Local Private",
    template: "single",
    networkMagic: 123,
    plan: {
      nodes: [],
      seedList: [],
      validatorsCount: 1,
      standbyCommittee: [],
    },
    status: "draft",
    createdAt: 1,
    ...overrides,
  };
}

function createSnapshot(): ConfigurationSnapshot {
  return {
    version: "private-network-plan-v1",
    nodes: [
      {
        name: "local-1",
        type: "neo-cli",
        network: "private",
      },
    ],
  };
}

describe("Actual private networks router", () => {
  let app: express.Application;
  let manager: {
    listPlans: ReturnType<typeof vi.fn>;
    getPlan: ReturnType<typeof vi.fn>;
    createPlan: ReturnType<typeof vi.fn>;
    buildConfigurationSnapshot: ReturnType<typeof vi.fn>;
    markApplied: ReturnType<typeof vi.fn>;
  };
  let nodeManager: {
    restoreConfiguration: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());

    manager = {
      listPlans: vi.fn(),
      getPlan: vi.fn(),
      createPlan: vi.fn(),
      buildConfigurationSnapshot: vi.fn(),
      markApplied: vi.fn(),
    };
    nodeManager = {
      restoreConfiguration: vi.fn(),
    };

    app.use("/api/private-networks", createPrivateNetworksRouter(manager, nodeManager));
  });

  it("lists private network plans", async () => {
    manager.listPlans.mockReturnValue([createPlan()]);

    const response = await request(app).get("/api/private-networks/plans");

    expect(response.status).toBe(200);
    expect(response.body.plans).toHaveLength(1);
  });

  it("creates a private network plan", async () => {
    const payload = { name: "Four Local", template: "four", networkMagic: 456 };
    manager.createPlan.mockReturnValue(createPlan({ id: "plan-2", template: "four", networkMagic: 456 }));

    const response = await request(app).post("/api/private-networks/plans").send(payload);

    expect(response.status).toBe(201);
    expect(response.body.plan.id).toBe("plan-2");
    expect(manager.createPlan).toHaveBeenCalledWith(payload);
  });

  it("supports the design-doc create plan path", async () => {
    const payload = { name: "Four Local", template: "four", networkMagic: 456 };
    manager.createPlan.mockReturnValue(createPlan({ id: "plan-2", template: "four", networkMagic: 456 }));

    const response = await request(app).post("/api/private-networks/plan").send(payload);

    expect(response.status).toBe(201);
    expect(response.body.plan.id).toBe("plan-2");
    expect(manager.createPlan).toHaveBeenCalledWith(payload);
  });

  it("gets a private network plan", async () => {
    manager.getPlan.mockReturnValue(createPlan());

    const response = await request(app).get("/api/private-networks/plans/plan-1");

    expect(response.status).toBe(200);
    expect(response.body.plan.id).toBe("plan-1");
    expect(manager.getPlan).toHaveBeenCalledWith("plan-1", { required: true });
  });

  it("returns a configuration snapshot", async () => {
    manager.buildConfigurationSnapshot.mockReturnValue(createSnapshot());

    const response = await request(app).get("/api/private-networks/plans/plan-1/configuration-snapshot");

    expect(response.status).toBe(200);
    expect(response.body.snapshot.version).toBe("private-network-plan-v1");
    expect(manager.buildConfigurationSnapshot).toHaveBeenCalledWith("plan-1");
  });

  it("applies a plan by restoring the generated snapshot and marking it applied on full success", async () => {
    const snapshot = createSnapshot();
    const appliedPlan = createPlan({ status: "applied", appliedAt: 99 });
    manager.buildConfigurationSnapshot.mockReturnValue(snapshot);
    nodeManager.restoreConfiguration.mockResolvedValue({ restoredCount: 1, skippedCount: 0, failedCount: 0 });
    manager.markApplied.mockReturnValue(appliedPlan);

    const response = await request(app)
      .post("/api/private-networks/plans/plan-1/apply")
      .send({ replaceExisting: true });

    expect(response.status).toBe(200);
    expect(nodeManager.restoreConfiguration).toHaveBeenCalledWith(snapshot, { replaceExisting: true });
    expect(manager.markApplied).toHaveBeenCalledWith("plan-1");
    expect(response.body).toMatchObject({
      result: { restoredCount: 1, skippedCount: 0, failedCount: 0 },
      plan: { status: "applied" },
    });
  });

  it("supports the design-doc apply path", async () => {
    const snapshot = createSnapshot();
    manager.buildConfigurationSnapshot.mockReturnValue(snapshot);
    nodeManager.restoreConfiguration.mockResolvedValue({ restoredCount: 1, skippedCount: 0, failedCount: 0 });
    manager.markApplied.mockReturnValue(createPlan({ status: "applied", appliedAt: 99 }));

    const response = await request(app).post("/api/private-networks/plan-1/apply").send({});

    expect(response.status).toBe(200);
    expect(nodeManager.restoreConfiguration).toHaveBeenCalledWith(snapshot, { replaceExisting: false });
    expect(manager.markApplied).toHaveBeenCalledWith("plan-1");
  });

  it("does not mark applied when restore reports failures", async () => {
    manager.buildConfigurationSnapshot.mockReturnValue(createSnapshot());
    nodeManager.restoreConfiguration.mockResolvedValue({ restoredCount: 0, skippedCount: 0, failedCount: 1 });

    const response = await request(app).post("/api/private-networks/plans/plan-1/apply").send({});

    expect(response.status).toBe(200);
    expect(manager.markApplied).not.toHaveBeenCalled();
    expect(response.body.plan).toBeNull();
  });

  it("does not mark applied when restore does not create every node", async () => {
    manager.buildConfigurationSnapshot.mockReturnValue(createSnapshot());
    nodeManager.restoreConfiguration.mockResolvedValue({ restoredCount: 0, skippedCount: 0, failedCount: 0 });

    const response = await request(app).post("/api/private-networks/plans/plan-1/apply").send({});

    expect(response.status).toBe(200);
    expect(manager.markApplied).not.toHaveBeenCalled();
    expect(response.body.plan).toBeNull();
  });

  it("rejects non-boolean replaceExisting before restore", async () => {
    manager.buildConfigurationSnapshot.mockReturnValue(createSnapshot());

    const response = await request(app)
      .post("/api/private-networks/plans/plan-1/apply")
      .send({ replaceExisting: "false" });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("PRIVATE_NETWORK_REPLACE_EXISTING_INVALID");
    expect(nodeManager.restoreConfiguration).not.toHaveBeenCalled();
    expect(manager.markApplied).not.toHaveBeenCalled();
  });

  it("returns structured errors for ApiError failures", async () => {
    manager.getPlan.mockImplementation(() => {
      throw new ApiError("PRIVATE_NETWORK_PLAN_NOT_FOUND", "Private network plan missing not found", "Create a plan first.", 404);
    });

    const response = await request(app).get("/api/private-networks/plans/missing");

    expect(response.status).toBe(404);
    expect(response.body).toMatchObject({
      code: "PRIVATE_NETWORK_PLAN_NOT_FOUND",
      error: "Private network plan missing not found",
      suggestion: "Create a plan first.",
    });
  });

  it("returns structured 400 errors for invalid create input", async () => {
    manager.createPlan.mockImplementation(() => {
      throw new ApiError("PRIVATE_NETWORK_PLAN_INVALID", "template must be single, four, or seven", "Choose a supported template.");
    });

    const response = await request(app).post("/api/private-networks/plans").send({ name: "Bad", template: "three" });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("PRIVATE_NETWORK_PLAN_INVALID");
  });
});
