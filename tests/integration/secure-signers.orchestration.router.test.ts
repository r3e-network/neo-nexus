import { beforeEach, describe, expect, it, vi } from "vitest";
import express from "express";
import request from "supertest";
import { createSecureSignersRouter } from "../../src/api/routes/secureSigners";

describe("Secure signer orchestration router", () => {
  let app: express.Application;
  let mockManager: {
    listProfiles: ReturnType<typeof vi.fn>;
    getProfile: ReturnType<typeof vi.fn>;
    createProfile: ReturnType<typeof vi.fn>;
    updateProfile: ReturnType<typeof vi.fn>;
    deleteProfile: ReturnType<typeof vi.fn>;
    testProfile: ReturnType<typeof vi.fn>;
    getOrchestration: ReturnType<typeof vi.fn>;
    getReadiness: ReturnType<typeof vi.fn>;
    fetchRecipientAttestation: ReturnType<typeof vi.fn>;
    startRecipientSigner: ReturnType<typeof vi.fn>;
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
      getOrchestration: vi.fn(),
      getReadiness: vi.fn(),
      fetchRecipientAttestation: vi.fn(),
      startRecipientSigner: vi.fn(),
    };

    app.use("/api/secure-signers", createSecureSignersRouter(mockManager as never));
  });

  it("returns orchestration helpers and readiness for a profile", async () => {
    mockManager.getOrchestration.mockResolvedValue({
      connection: { scheme: "vsock", servicePort: 9991, startupPort: 9992, cid: 2345 },
      commands: {
        deploy: ["./scripts/nitro/run.sh --cid 2345 --eif-path secure-sign-nitro.eif"],
        unlock: ["./scripts/auto-unlock-kms-recipient.sh"],
        status: ["./target/secure-sign-tools status --cid 2345 --port 9991 --public-key <pubkey>"],
      },
      warnings: [],
    });
    mockManager.getReadiness.mockResolvedValue({
      ok: true,
      status: "reachable",
      source: "secure-sign-tools",
      accountStatus: "Single",
    });

    const response = await request(app).get("/api/secure-signers/signer-1/orchestration");

    expect(response.status).toBe(200);
    expect(response.body.orchestration.connection.cid).toBe(2345);
    expect(response.body.orchestration.readiness.accountStatus).toBe("Single");
  });

  it("fetches Nitro recipient attestation", async () => {
    mockManager.fetchRecipientAttestation.mockResolvedValue({
      attestationBase64: "QVRURVNUQVRJT05fRE9D",
      checkedAt: Date.now(),
    });

    const response = await request(app).post("/api/secure-signers/signer-1/attestation");

    expect(response.status).toBe(200);
    expect(response.body.attestation.attestationBase64).toBe("QVRURVNUQVRJT05fRE9D");
  });

  it("starts a signer with recipient ciphertext", async () => {
    mockManager.startRecipientSigner.mockResolvedValue({
      ok: true,
      message: "Signer starting via recipient ciphertext...",
      checkedAt: Date.now(),
    });

    const response = await request(app)
      .post("/api/secure-signers/signer-1/start-recipient")
      .send({ ciphertextBase64: "ciphertext-value" });

    expect(response.status).toBe(200);
    expect(mockManager.startRecipientSigner).toHaveBeenCalledWith("signer-1", "ciphertext-value");
  });
});
