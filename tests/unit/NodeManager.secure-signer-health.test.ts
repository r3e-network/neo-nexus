import { describe, expect, it, vi } from "vitest";
import { NodeManager } from "../../src/core/NodeManager";

describe("NodeManager secure signer health", () => {
  it("returns null when a node does not use secure signer protection", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      secureSignerManager: {
        getProfile: ReturnType<typeof vi.fn>;
        getReadiness: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => ({
      id: "node-1",
      settings: {},
    }));
    manager.secureSignerManager = {
      getProfile: vi.fn(),
      getReadiness: vi.fn(),
    };

    const health = await (manager as any).getNodeSecureSignerHealth("node-1");
    expect(health).toBeNull();
  });

  it("returns secure signer readiness for a bound node", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
      secureSignerManager: {
        getProfile: ReturnType<typeof vi.fn>;
        getReadiness: ReturnType<typeof vi.fn>;
      };
    };

    manager.getNode = vi.fn(() => ({
      id: "node-1",
      name: "Consensus Node",
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
        },
      },
    }));
    manager.secureSignerManager = {
      getProfile: vi.fn(() => ({
        id: "signer-1",
        name: "Nitro Council",
        mode: "nitro",
        endpoint: "vsock://2345:9991",
        enabled: true,
      })),
      getReadiness: vi.fn(async () => ({
        ok: true,
        status: "reachable",
        source: "secure-sign-tools",
        accountStatus: "Single",
      })),
    };

    const health = await (manager as any).getNodeSecureSignerHealth("node-1");

    expect(health).toEqual({
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
      },
    });
  });
});
