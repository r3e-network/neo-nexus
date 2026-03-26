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

      if (sql.includes("DELETE FROM secure_signer_profiles WHERE id = ?")) {
        return {
          run: vi.fn((id: string) => {
            const index = profiles.findIndex((entry) => entry.id === id);
            if (index > -1) {
              profiles.splice(index, 1);
            }
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

describe("SecureSignerManager", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("creates and lists normalized secure signer profiles", async () => {
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never, {
      probeEndpoint: vi.fn(async () => ({ ok: true, message: "connected", latencyMs: 12 })),
    });

    const created = manager.createProfile({
      name: "  Nitro Council Signer  ",
      mode: "nitro",
      endpoint: "vsock://2345:9991",
      publicKey: "03b209fd4f53a7170ea4444e0cb0a6bb6a53c2bd016926989cf85f9b0fba17a70c",
      unlockMode: "recipient-attestation",
    });

    expect(created.name).toBe("Nitro Council Signer");
    expect(created.endpoint).toBe("vsock://2345:9991");
    expect(created.enabled).toBe(true);
    expect(manager.listProfiles()).toEqual([created]);
  });

  it("rejects invalid endpoint and mode combinations", async () => {
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never);

    expect(() =>
      manager.createProfile({
        name: "Broken Nitro",
        mode: "nitro",
        endpoint: "http://127.0.0.1:9991",
      }),
    ).toThrow(/vsock/i);

    expect(() =>
      manager.createProfile({
        name: "Broken SGX",
        mode: "sgx",
        endpoint: "vsock://1234:9991",
      }),
    ).toThrow(/http|https/i);
  });

  it("tests profiles and records warning status for vsock endpoints", async () => {
    const probeEndpoint = vi.fn(async () => ({ ok: true, message: "connected", latencyMs: 18 }));
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never, { probeEndpoint });

    const httpProfile = manager.createProfile({
      name: "Local Mock Signer",
      mode: "software",
      endpoint: "http://127.0.0.1:9991",
    });

    const nitroProfile = manager.createProfile({
      name: "Nitro Signer",
      mode: "nitro",
      endpoint: "vsock://2345:9991",
    });

    const httpResult = await manager.testProfile(httpProfile.id);
    expect(httpResult.ok).toBe(true);
    expect(httpResult.status).toBe("reachable");
    expect(probeEndpoint).toHaveBeenCalledWith("http://127.0.0.1:9991");

    const nitroResult = await manager.testProfile(nitroProfile.id);
    expect(nitroResult.ok).toBe(true);
    expect(nitroResult.status).toBe("warning");
    expect(nitroResult.message).toMatch(/vsock/i);
  });
});
