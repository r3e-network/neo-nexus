import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { ApiError, Errors } from "../../src/api/errors";
import { createNodeRoleApplicationsRouter, createNodeRolesRouter } from "../../src/api/routes/nodeRoles";
import type { NodeInstance, NodeRoleApplication, NodeRoleApplicationPlan, NodeRoleProfile } from "../../src/types";

function createRole(overrides: Partial<NodeRoleProfile> = {}): NodeRoleProfile {
  return {
    id: "builtin-state",
    name: "State Node",
    description: "State role",
    kind: "builtin",
    nodeTypes: ["neo-cli"],
    profile: { storageEngine: "rocksdb" },
    createdAt: 0,
    updatedAt: 0,
    ...overrides,
  };
}

function createNode(overrides: Partial<NodeInstance> = {}): NodeInstance {
  return {
    id: "node-1",
    name: "Node 1",
    chain: "n3",
    type: "neo-cli",
    network: "mainnet",
    syncMode: "full",
    version: "3.9.2",
    ports: { rpc: 10332, p2p: 10333 },
    paths: { base: "/tmp/node-1", data: "/tmp/node-1/data", logs: "/tmp/node-1/logs", config: "/tmp/node-1/config" },
    settings: {},
    createdAt: 1,
    updatedAt: 1,
    process: { status: "stopped" },
    plugins: [],
    ...overrides,
  };
}

function createPlan(overrides: Partial<NodeRoleApplicationPlan> = {}): NodeRoleApplicationPlan {
  return {
    nodeId: "node-1",
    roleId: "builtin-state",
    roleName: "State Node",
    requiresRestart: true,
    changes: [{ type: "storage", summary: "Switch storage engine to rocksdb" }],
    warnings: [],
    ...overrides,
  };
}

function createApplication(overrides: Partial<NodeRoleApplication> = {}): NodeRoleApplication {
  return {
    id: "role-app-1",
    nodeId: "node-1",
    roleId: "builtin-state",
    roleName: "State Node",
    applicationPlan: createPlan(),
    appliedAt: 99,
    status: "applied",
    ...overrides,
  };
}

describe("Actual node roles router", () => {
  let app: express.Application;
  let roleManager: {
    listRoles: ReturnType<typeof vi.fn>;
    getRole: ReturnType<typeof vi.fn>;
    createCustomRole: ReturnType<typeof vi.fn>;
    listApplications: ReturnType<typeof vi.fn>;
  };
  let applicationService: {
    plan: ReturnType<typeof vi.fn>;
    apply: ReturnType<typeof vi.fn>;
  };
  let nodeManager: {
    getNode: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());
    app.use((req, _res, next) => {
      req.user = { id: "admin-1", username: "admin", role: "admin" };
      next();
    });

    roleManager = {
      listRoles: vi.fn(() => [createRole()]),
      getRole: vi.fn(() => createRole()),
      createCustomRole: vi.fn((input) => createRole({ id: "role-custom", kind: "custom", ...input })),
      listApplications: vi.fn(() => [createApplication()]),
    };
    applicationService = {
      plan: vi.fn(() => createPlan()),
      apply: vi.fn(async () => ({ application: createApplication(), node: createNode({ settings: { storageEngine: "rocksdb" } }) })),
    };
    nodeManager = {
      getNode: vi.fn(() => createNode()),
    };

    app.use("/api/node-roles", createNodeRolesRouter({
      roleManager: roleManager as never,
      applicationService: applicationService as never,
    }));
    app.use("/api/nodes/:id/role-applications", createNodeRoleApplicationsRouter({
      nodeManager: nodeManager as never,
      roleManager: roleManager as never,
    }));
  });

  it("lists roles", async () => {
    const response = await request(app).get("/api/node-roles");

    expect(response.status).toBe(200);
    expect(response.body.roles).toHaveLength(1);
  });

  it("gets a role by id", async () => {
    const response = await request(app).get("/api/node-roles/builtin-state");

    expect(response.status).toBe(200);
    expect(response.body.role.id).toBe("builtin-state");
    expect(roleManager.getRole).toHaveBeenCalledWith("builtin-state");
  });

  it("returns a structured 404 when a role is missing", async () => {
    roleManager.getRole.mockReturnValueOnce(null);

    const response = await request(app).get("/api/node-roles/missing");

    expect(response.status).toBe(404);
    expect(response.body).toMatchObject({
      code: "NODE_ROLE_NOT_FOUND",
      status: 404,
    });
  });

  it("creates a custom role with the authenticated user id", async () => {
    const payload = {
      name: "RPC Custom",
      description: "Custom RPC",
      nodeTypes: ["neo-cli"],
      profile: { plugins: [{ id: "RpcServer", enabled: true }] },
    };

    const response = await request(app).post("/api/node-roles").send(payload);

    expect(response.status).toBe(201);
    expect(response.body.role.id).toBe("role-custom");
    expect(roleManager.createCustomRole).toHaveBeenCalledWith({ ...payload, createdBy: "admin-1" });
  });

  it("returns structured 400 for invalid custom role input", async () => {
    const response = await request(app).post("/api/node-roles").send({ name: "Missing bits" });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("MISSING_FIELDS");
    expect(roleManager.createCustomRole).not.toHaveBeenCalled();
  });

  it("rejects custom roles with unknown plugin ids before persistence", async () => {
    const response = await request(app).post("/api/node-roles").send({
      name: "Bad Plugin",
      nodeTypes: ["neo-cli"],
      profile: {
        plugins: [{ id: "UnknownPlugin", enabled: true }],
      },
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("NODE_ROLE_REQUEST_INVALID");
    expect(roleManager.createCustomRole).not.toHaveBeenCalled();
  });

  it("rejects plugin-bearing custom roles for non neo-cli node types", async () => {
    const response = await request(app).post("/api/node-roles").send({
      name: "Bad Neo Go Plugin Role",
      nodeTypes: ["neo-go"],
      profile: {
        plugins: [{ id: "RpcServer", enabled: true }],
      },
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("NODE_ROLE_REQUEST_INVALID");
    expect(roleManager.createCustomRole).not.toHaveBeenCalled();
  });

  it("rejects custom roles with invalid sync or data context shape", async () => {
    const response = await request(app).post("/api/node-roles").send({
      name: "Bad Sync",
      nodeTypes: ["neo-cli"],
      profile: {
        sync: { strategy: "warp" },
        dataContext: { mode: "reuse", labelTemplate: "" },
      },
    });

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("NODE_ROLE_REQUEST_INVALID");
    expect(roleManager.createCustomRole).not.toHaveBeenCalled();
  });

  it("plans role application", async () => {
    const response = await request(app).post("/api/node-roles/builtin-state/plan").send({
      nodeId: "node-1",
      storageEngine: "rocksdb",
    });

    expect(response.status).toBe(200);
    expect(response.body.plan.roleId).toBe("builtin-state");
    expect(applicationService.plan).toHaveBeenCalledWith("builtin-state", "node-1", { storageEngine: "rocksdb" });
  });

  it("returns structured 400 for missing plan body fields", async () => {
    const response = await request(app).post("/api/node-roles/builtin-state/plan").send({});

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("MISSING_FIELDS");
    expect(applicationService.plan).not.toHaveBeenCalled();
  });

  it("applies a role", async () => {
    const response = await request(app).post("/api/node-roles/builtin-state/apply").send({
      nodeId: "node-1",
    });

    expect(response.status).toBe(200);
    expect(response.body.application.status).toBe("applied");
    expect(response.body.node.settings.storageEngine).toBe("rocksdb");
    expect(applicationService.apply).toHaveBeenCalledWith("builtin-state", "node-1", {}, "admin-1");
  });

  it("passes ApiError failures through from apply", async () => {
    applicationService.apply.mockRejectedValueOnce(Errors.nodeRunning());

    const response = await request(app).post("/api/node-roles/builtin-state/apply").send({ nodeId: "node-1" });

    expect(response.status).toBe(409);
    expect(response.body.code).toBe("NODE_RUNNING");
  });

  it("lists role application history for a node", async () => {
    const response = await request(app).get("/api/nodes/node-1/role-applications");

    expect(response.status).toBe(200);
    expect(response.body.applications).toHaveLength(1);
    expect(nodeManager.getNode).toHaveBeenCalledWith("node-1");
    expect(roleManager.listApplications).toHaveBeenCalledWith("node-1");
  });

  it("returns node-not-found for application history on a missing node", async () => {
    nodeManager.getNode.mockReturnValueOnce(null);

    const response = await request(app).get("/api/nodes/missing/role-applications");

    expect(response.status).toBe(404);
    expect(response.body.code).toBe("NODE_NOT_FOUND");
  });

  it("does not turn ApiError role lookup failures into 500", async () => {
    roleManager.getRole.mockImplementationOnce(() => {
      throw new ApiError("NODE_ROLE_CORRUPT", "Role profile is corrupt", "Recreate the role.");
    });

    const response = await request(app).get("/api/node-roles/builtin-state");

    expect(response.status).toBe(400);
    expect(response.body.code).toBe("NODE_ROLE_CORRUPT");
  });
});
