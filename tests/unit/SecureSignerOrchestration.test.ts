import { beforeEach, describe, expect, it, vi } from "vitest";

function createMockDb() {
  const profiles: any[] = [];

  return {
    prepare: vi.fn((sql: string) => {
      if (sql.includes("SELECT * FROM secure_signer_profiles WHERE id = ?")) {
        return {
          get: vi.fn((id: string) => profiles.find((profile) => profile.id === id)),
        };
      }

      if (sql.includes("SELECT * FROM secure_signer_profiles ORDER BY created_at")) {
        return {
          all: vi.fn(() => [...profiles]),
        };
      }

      if (sql.includes("INSERT INTO secure_signer_profiles")) {
        return {
          run: vi.fn((...args: any[]) => {
            profiles.push({
              id: args[0],
              name: args[1],
              mode: args[2],
              endpoint: args[3],
              public_key: args[4],
              account_address: args[5],
              wallet_path: args[6],
              unlock_mode: args[7],
              notes: args[8],
              enabled: args[9],
              workspace_path: args[10],
              startup_port: args[11],
              aws_region: args[12],
              kms_key_id: args[13],
              kms_ciphertext_blob_path: args[14],
              last_test_status: null,
              last_test_message: null,
              last_tested_at: null,
              created_at: args[15],
              updated_at: args[16],
            });
          }),
        };
      }

      if (sql.includes("UPDATE secure_signer_profiles") && sql.includes("last_test_status")) {
        return {
          run: vi.fn((status: string, message: string | null, testedAt: number, id: string) => {
            const profile = profiles.find((entry) => entry.id === id);
            if (profile) {
              profile.last_test_status = status;
              profile.last_test_message = message;
              profile.last_tested_at = testedAt;
            }
          }),
        };
      }

      if (sql.includes("UPDATE secure_signer_profiles")) {
        return {
          run: vi.fn((...args: any[]) => {
            const id = args[args.length - 1];
            const profile = profiles.find((entry) => entry.id === id);
            if (!profile) return;

            profile.name = args[0];
            profile.mode = args[1];
            profile.endpoint = args[2];
            profile.public_key = args[3];
            profile.account_address = args[4];
            profile.wallet_path = args[5];
            profile.unlock_mode = args[6];
            profile.notes = args[7];
            profile.enabled = args[8];
            profile.workspace_path = args[9];
            profile.startup_port = args[10];
            profile.aws_region = args[11];
            profile.kms_key_id = args[12];
            profile.kms_ciphertext_blob_path = args[13];
            profile.updated_at = args[14];
          }),
        };
      }

      return {
        get: vi.fn(),
        all: vi.fn(() => []),
        run: vi.fn(),
      };
    }),
  };
}

describe("SecureSignerManager orchestration", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("builds Nitro lifecycle commands from profile metadata", async () => {
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never, {
      probeEndpoint: vi.fn(async () => ({ ok: true, message: "connected" })),
      runToolCommand: vi.fn(),
    } as never);

    const profile = manager.createProfile({
      name: "Nitro Council",
      mode: "nitro",
      endpoint: "vsock://2345:9991",
      publicKey: "03b209fd4f53a7170ea4444e0cb0a6bb6a53c2bd016926989cf85f9b0fba17a70c",
      workspacePath: "/opt/secure-sign-service-rs",
      awsRegion: "ap-southeast-1",
      kmsKeyId: "kms-key-123",
      kmsCiphertextBlobPath: "/secure/wallet-passphrase.kms.bin",
    });

    const orchestration = manager.getOrchestration(profile.id);

    expect(orchestration.connection).toMatchObject({
      scheme: "vsock",
      servicePort: 9991,
      startupPort: 9992,
      cid: 2345,
    });
    expect(orchestration.commands.deploy[0]).toContain("./scripts/nitro/run.sh");
    expect(orchestration.commands.unlock[0]).toContain("auto-unlock-kms-recipient.sh");
    expect(orchestration.commands.status[0]).toContain("secure-sign-tools status");
  });

  it("reads signer readiness through secure-sign-tools status when local tooling is available", async () => {
    const runToolCommand = vi.fn(async () => ({
      stdout:
        "Account 03b209fd4f53a7170ea4444e0cb0a6bb6a53c2bd016926989cf85f9b0fba17a70c status: Single\n",
      stderr: "",
    }));
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never, {
      probeEndpoint: vi.fn(async () => ({ ok: true, message: "connected" })),
      runToolCommand,
    } as never);

    const profile = manager.createProfile({
      name: "Local Mock",
      mode: "software",
      endpoint: "http://127.0.0.1:9991",
      publicKey: "03b209fd4f53a7170ea4444e0cb0a6bb6a53c2bd016926989cf85f9b0fba17a70c",
      workspacePath: "/opt/secure-sign-service-rs",
    });

    const readiness = await manager.getReadiness(profile.id);

    expect(readiness.ok).toBe(true);
    expect(readiness.accountStatus).toBe("Single");
    expect(readiness.source).toBe("secure-sign-tools");
    expect(runToolCommand).toHaveBeenCalled();
  });

  it("fetches Nitro recipient attestation and starts the signer with recipient ciphertext", async () => {
    const runToolCommand = vi
      .fn()
      .mockResolvedValueOnce({
        stdout: "QVRURVNUQVRJT05fRE9D",
        stderr: "",
      })
      .mockResolvedValueOnce({
        stdout: "Signer starting via recipient ciphertext...",
        stderr: "",
      });

    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never, {
      probeEndpoint: vi.fn(async () => ({ ok: true, message: "connected" })),
      runToolCommand,
    } as never);

    const profile = manager.createProfile({
      name: "Nitro Council",
      mode: "nitro",
      endpoint: "vsock://2345:9991",
      publicKey: "03b209fd4f53a7170ea4444e0cb0a6bb6a53c2bd016926989cf85f9b0fba17a70c",
      workspacePath: "/opt/secure-sign-service-rs",
      awsRegion: "ap-southeast-1",
    });

    const attestation = await manager.fetchRecipientAttestation(profile.id);
    expect(attestation.attestationBase64).toBe("QVRURVNUQVRJT05fRE9D");

    const startResult = await manager.startRecipientSigner(profile.id, "ciphertext-base64-value");
    expect(startResult.ok).toBe(true);
    expect(runToolCommand).toHaveBeenNthCalledWith(
      2,
      "/opt/secure-sign-service-rs/target/secure-sign-tools",
      ["start-recipient", "--cid", "2345", "--port", "9992", "--ciphertext-base64", "ciphertext-base64-value"],
    );
  });
});
