import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { createSecureSignersRouter } from "../../src/api/routes/secureSigners";

describe("Actual secure signers router", () => {
  let app: express.Application;
  let mockManager: {
    listProfiles: ReturnType<typeof vi.fn>;
    getProfile: ReturnType<typeof vi.fn>;
    createProfile: ReturnType<typeof vi.fn>;
    updateProfile: ReturnType<typeof vi.fn>;
    deleteProfile: ReturnType<typeof vi.fn>;
    testProfile: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    app = express();
    app.use(express.json());

    mockManager = {
      listProfiles: vi.fn(),
      getProfile: vi.fn(),
      createProfile: vi.fn(),
      updateProfile: vi.fn(),
      deleteProfile: vi.fn(),
      testProfile: vi.fn(),
    };

    app.use("/api/secure-signers", createSecureSignersRouter(mockManager as never));
  });

  it("lists secure signer profiles", async () => {
    mockManager.listProfiles.mockReturnValue([
      { id: "signer-1", name: "Nitro Council", mode: "nitro", endpoint: "vsock://2345:9991" },
    ]);

    const response = await request(app).get("/api/secure-signers");

    expect(response.status).toBe(200);
    expect(response.body.profiles).toHaveLength(1);
  });

  it("returns a single secure signer profile", async () => {
    mockManager.getProfile.mockReturnValue({
      id: "signer-1",
      name: "SGX Signer",
      mode: "sgx",
      endpoint: "https://sgx.example.com:9443",
    });

    const response = await request(app).get("/api/secure-signers/signer-1");

    expect(response.status).toBe(200);
    expect(response.body.profile.id).toBe("signer-1");
  });

  it("creates a secure signer profile", async () => {
    mockManager.createProfile.mockReturnValue({
      id: "signer-1",
      name: "Local Mock",
      mode: "software",
      endpoint: "http://127.0.0.1:9991",
      enabled: true,
    });

    const response = await request(app).post("/api/secure-signers").send({
      name: "Local Mock",
      mode: "software",
      endpoint: "http://127.0.0.1:9991",
    });

    expect(response.status).toBe(201);
    expect(response.body.profile.id).toBe("signer-1");
  });

  it("updates a secure signer profile", async () => {
    mockManager.updateProfile.mockReturnValue({
      id: "signer-1",
      name: "Nitro Signer Updated",
      mode: "nitro",
      endpoint: "vsock://2345:9991",
    });

    const response = await request(app).put("/api/secure-signers/signer-1").send({
      name: "Nitro Signer Updated",
    });

    expect(response.status).toBe(200);
    expect(response.body.profile.name).toBe("Nitro Signer Updated");
  });

  it("tests a secure signer profile", async () => {
    mockManager.testProfile.mockResolvedValue({
      ok: true,
      status: "warning",
      message: "Vsock endpoints cannot be probed directly.",
      checkedAt: Date.now(),
    });

    const response = await request(app).post("/api/secure-signers/signer-1/test");

    expect(response.status).toBe(200);
    expect(response.body.result.status).toBe("warning");
  });

  it("returns structured error when profile is not found", async () => {
    mockManager.getProfile.mockReturnValue(null);

    const response = await request(app).get("/api/secure-signers/nonexistent");

    expect(response.status).toBe(404);
    expect(response.body.error).toBe("Secure signer profile not found");
    expect(response.body.code).toBe("SIGNER_NOT_FOUND");
    expect(response.body.suggestion).toBeDefined();
  });

  it("returns structured error when required fields are missing on create", async () => {
    const response = await request(app).post("/api/secure-signers").send({
      name: "Incomplete",
    });

    expect(response.status).toBe(400);
    expect(response.body.error).toBe("Missing required fields: name, mode, endpoint");
    expect(response.body.code).toBe("MISSING_FIELDS");
    expect(response.body.suggestion).toBeDefined();
  });

  it("deletes a secure signer profile", async () => {
    mockManager.deleteProfile.mockReturnValue(undefined);

    const response = await request(app).delete("/api/secure-signers/signer-1");

    expect(response.status).toBe(204);
  });
});
