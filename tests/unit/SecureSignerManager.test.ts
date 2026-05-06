import { createServer as createTcpServer, type Server as TcpServer } from "node:net";
import { beforeEach, describe, expect, it, vi } from "vitest";

function createMockDb() {
  const profiles: any[] = [];
  const auditRows: any[] = [];

  return {
    auditRows,
    prepare: vi.fn((sql: string) => {
      if (sql.includes("INSERT INTO audit_log")) {
        return {
          run: vi.fn((timestamp: number, action: string, resourceType: string, resourceId: string, details: string) => {
            auditRows.push({ timestamp, action, resourceType, resourceId, details: JSON.parse(details) });
          }),
        };
      }

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

async function startTcpServer() {
  const server = createTcpServer((socket) => {
    socket.end();
  });
  const port = await new Promise<number>((resolve) => {
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (typeof address === "object" && address) {
        resolve(address.port);
      }
    });
  });
  return {
    port,
    close: () => closeTcpServer(server),
  };
}

function closeTcpServer(server: TcpServer): Promise<void> {
  return new Promise((resolve, reject) => {
    server.close((error) => {
      if (error) reject(error);
      else resolve();
    });
  });
}

describe("SecureSignerManager", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    delete process.env.NEONEXUS_ALLOW_PRIVATE_SIGNER_ENDPOINTS;
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

  it("rejects secure signer startup ports outside the valid TCP range", async () => {
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never);

    expect(() =>
      manager.createProfile({
        name: "Nitro",
        mode: "nitro",
        endpoint: "vsock://2345:9991",
        startupPort: 70000,
      }),
    ).toThrow(/1 and 65535/i);

    expect(() =>
      manager.createProfile({
        name: "Software",
        mode: "software",
        endpoint: "https://signer.example.com:0",
      }),
    ).toThrow(/1 and 65535/i);

    expect(() =>
      manager.createProfile({
        name: "Nitro",
        mode: "nitro",
        endpoint: "vsock://2345:65535",
      }),
    ).toThrow(/startup port/i);
  });

  it("tests profiles and records warning status for vsock endpoints", async () => {
    process.env.NEONEXUS_ALLOW_PRIVATE_SIGNER_ENDPOINTS = "true";
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

  it("blocks private and local HTTP signer endpoints by default", async () => {
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never);

    expect(() => manager.createProfile({
      name: "Loopback signer",
      mode: "software",
      endpoint: "http://127.0.0.1:9991",
    })).toThrow(/private or local/i);

    expect(() => manager.createProfile({
      name: "Metadata signer",
      mode: "sgx",
      endpoint: "http://169.254.169.254:9991",
    })).toThrow(/private or local/i);

    expect(() => manager.createProfile({
      name: "IPv4 mapped metadata signer",
      mode: "software",
      endpoint: "http://[::ffff:169.254.169.254]:9991",
    })).toThrow(/private or local/i);
  });

  it("allows private HTTP signer endpoints only when explicitly enabled", async () => {
    process.env.NEONEXUS_ALLOW_PRIVATE_SIGNER_ENDPOINTS = "true";
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never);

    const profile = manager.createProfile({
      name: "Local signer",
      mode: "software",
      endpoint: "http://127.0.0.1:9991",
    });

    expect(profile.endpoint).toBe("http://127.0.0.1:9991");
  });

  it("reports DNS-private signer endpoints unreachable before probing", async () => {
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never, {
      resolveHostname: vi.fn(async () => [{ address: "127.0.0.1", family: 4 }]),
    });

    const profile = manager.createProfile({
      name: "DNS private signer",
      mode: "software",
      endpoint: "https://signer-private.example.com:9991",
    });

    const result = await manager.testProfile(profile.id);

    expect(result.ok).toBe(false);
    expect(result.status).toBe("unreachable");
    expect(result.message).toMatch(/private or local/i);
  });

  it("connects signer probes to the already-resolved address when private targets are explicitly enabled", async () => {
    process.env.NEONEXUS_ALLOW_PRIVATE_SIGNER_ENDPOINTS = "true";
    const server = await startTcpServer();
    const resolveHostname = vi.fn(async () => [{ address: "127.0.0.1", family: 4 as const }]);
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never, { resolveHostname });
    const profile = manager.createProfile({
      name: "Pinned signer",
      mode: "software",
      endpoint: `http://signer.example.test:${server.port}`,
    });

    try {
      const result = await manager.testProfile(profile.id);

      expect(result.ok).toBe(true);
      expect(result.status).toBe("reachable");
      expect(resolveHostname).toHaveBeenCalledWith("signer.example.test");
    } finally {
      await server.close();
    }
  });

  it("declares signer capabilities without overstating software isolation", async () => {
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never);

    const software = manager.createProfile({
      name: "Software fallback",
      mode: "software",
      endpoint: "https://signer.example.com:9991",
    });
    const nitro = manager.createProfile({
      name: "Nitro",
      mode: "nitro",
      endpoint: "vsock://2345:9991",
      publicKey: "03b209fd4f53a7170ea4444e0cb0a6bb6a53c2bd016926989cf85f9b0fba17a70c",
    });

    expect(manager.getCapabilityDeclaration(software.id)).toMatchObject({
      isolation: "software-fallback",
      privateKeyExportable: false,
      hardwareBacked: false,
      experimental: true,
    });
    expect(manager.getCapabilityDeclaration(nitro.id)).toMatchObject({
      isolation: "nitro-enclave",
      privateKeyExportable: false,
      hardwareBacked: true,
      remoteAttestation: true,
    });
  });

  it("embeds an explicit fail-closed SignClient policy even when no allowlist is supplied", async () => {
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never);
    const profile = manager.createProfile({
      name: "Nitro",
      mode: "nitro",
      endpoint: "vsock://2345:9991",
      publicKey: "03b209fd4f53a7170ea4444e0cb0a6bb6a53c2bd016926989cf85f9b0fba17a70c",
    });

    expect(manager.buildSignClientConfig(profile)).toEqual({
      Name: "Nitro",
      Endpoint: "vsock://2345:9991",
      Policy: {
        RequireHardwareProtection: false,
        DenyByDefault: true,
      },
    });
  });

  it("embeds an explicit allowlist policy in SignClient config", async () => {
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never);
    const profile = manager.createProfile({
      name: "Nitro",
      mode: "nitro",
      endpoint: "vsock://2345:9991",
      publicKey: "03b209fd4f53a7170ea4444e0cb0a6bb6a53c2bd016926989cf85f9b0fba17a70c",
    });

    expect(manager.buildSignClientConfig(profile, {
      allowedOperations: ["vote", "oracle-response"],
      allowedContracts: ["0x1234567890abcdef1234567890abcdef12345678"],
      allowedRecipients: ["NXV7ZhHiyM1aHXwpVsRZC6BwNFP2jghXAq"],
      requireHardwareProtection: true,
    })).toEqual({
      Name: "Nitro",
      Endpoint: "vsock://2345:9991",
      Policy: {
        AllowedOperations: ["vote", "oracle-response"],
        AllowedContracts: ["0x1234567890abcdef1234567890abcdef12345678"],
        AllowedRecipients: ["NXV7ZhHiyM1aHXwpVsRZC6BwNFP2jghXAq"],
        RequireHardwareProtection: true,
        DenyByDefault: true,
      },
    });
  });

  it("blocks hardware-required policies for software signers and writes an audit trail", async () => {
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const db = createMockDb();
    const manager = new SecureSignerManager(db as never);
    const profile = manager.createProfile({
      name: "Software fallback",
      mode: "software",
      endpoint: "https://signer.example.com:9991",
    });

    expect(() => manager.buildSignClientConfig(profile, { requireHardwareProtection: true })).toThrow(/hardware-backed/i);
    expect(db.auditRows).toEqual(expect.arrayContaining([
      expect.objectContaining({
        action: "secure-signer.policy.block",
        resourceId: profile.id,
        details: expect.objectContaining({ reason: "hardware-protection-required" }),
      }),
    ]));
  });

  it("rejects signer workspace paths outside approved roots", async () => {
    const { SecureSignerManager } = await import("../../src/core/SecureSignerManager");
    const manager = new SecureSignerManager(createMockDb() as never, {
      probeEndpoint: vi.fn(),
      runToolCommand: vi.fn(),
    });

    expect(() => manager.createProfile({
      name: "Injected Workspace",
      mode: "software",
      endpoint: "https://signer.example.com:9991",
      workspacePath: "/tmp/attacker-controlled",
    })).toThrow(/workspace path is not allowed/i);
  });
});
