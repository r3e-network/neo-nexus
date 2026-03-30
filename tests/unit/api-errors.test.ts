import { describe, expect, it } from "vitest";
import { ApiError, Errors } from "../../src/api/errors.ts";

describe("ApiError", () => {
  it("carries code, message, suggestion, and status", () => {
    const err = new ApiError("NODE_NOT_FOUND", "Node not found",
      "Check the node ID.", 404);

    expect(err).toBeInstanceOf(Error);
    expect(err.code).toBe("NODE_NOT_FOUND");
    expect(err.message).toBe("Node not found");
    expect(err.suggestion).toBe("Check the node ID.");
    expect(err.status).toBe(404);
  });

  it("defaults status to 400", () => {
    const err = new ApiError("MISSING_FIELDS", "Missing fields", "Provide all fields.");
    expect(err.status).toBe(400);
  });
});

describe("Errors catalog", () => {
  it("nodeNotFound returns 404 with suggestion", () => {
    const err = Errors.nodeNotFound("node-abc");
    expect(err.code).toBe("NODE_NOT_FOUND");
    expect(err.message).toBe("Node node-abc not found");
    expect(err.status).toBe(404);
    expect(err.suggestion).toContain("GET /api/nodes");
  });

  it("nodeNotFound works without an id", () => {
    const err = Errors.nodeNotFound();
    expect(err.message).toBe("Node not found");
  });

  it("missingFields lists the required fields", () => {
    const err = Errors.missingFields("name", "type", "network");
    expect(err.code).toBe("MISSING_FIELDS");
    expect(err.message).toContain("name, type, network");
    expect(err.status).toBe(400);
  });

  it("every factory returns an ApiError", () => {
    const factories = [
      () => Errors.nodeNotFound(),
      () => Errors.nodeRunning(),
      () => Errors.nodeAlreadyRunning(),
      () => Errors.nodeNotRunning(),
      () => Errors.missingFields("a"),
      () => Errors.missingField("path"),
      () => Errors.nameExists("x"),
      () => Errors.invalidCredentials(),
      () => Errors.noToken(),
      () => Errors.tokenInvalid(),
      () => Errors.sessionInvalid(),
      () => Errors.pathBlocked("/etc"),
      () => Errors.pathNotAllowed(),
      () => Errors.pathNotFound("/x"),
      () => Errors.detectionFailed("/x"),
      () => Errors.detectionNotFound(),
      () => Errors.importInvalid(["bad port"]),
      () => Errors.signerRequiresProfile(),
      () => Errors.signerNeoCliOnly(),
      () => Errors.signerNotAvailable("s1"),
      () => Errors.signerProfileNotFound(),
      () => Errors.signerFieldsRequired(),
      () => Errors.portConflictNode(10332, "RPC"),
      () => Errors.portConflictSystem(10332, "RPC"),
      () => Errors.noPortRange(),
      () => Errors.pluginsCliOnly(),
      () => Errors.setupCompleted(),
      () => Errors.credentialsRequired(),
      () => Errors.adminRequired(),
      () => Errors.passwordRequired(),
      () => Errors.cannotDeleteSelf(),
      () => Errors.notAuthenticated(),
      () => Errors.serverFieldsRequired(),
      () => Errors.snapshotRequired(),
    ];

    for (const factory of factories) {
      const err = factory();
      expect(err).toBeInstanceOf(ApiError);
      expect(err.code).toBeTruthy();
      expect(err.suggestion).toBeTruthy();
    }
  });
});
