import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../src/core/DownloadManager", () => ({
  DownloadManager: {
    getNodeBinaryPath: vi.fn(() => null),
  },
}));

import { ConfigManager } from "../../src/core/ConfigManager";
import { DownloadManager } from "../../src/core/DownloadManager";
import type { NodeConfig } from "../../src/types";

const privateNetwork = {
  networkMagic: 987654,
  validatorsCount: 4,
  standbyCommittee: [
    "02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "03bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  ],
  seedList: ["127.0.0.1:30333", "127.0.0.1:30343"],
  publicKey: "02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  address: "NNLi44dJNXtDNSBkofB48aTVYtb1zZrNEs",
};

const tempDirs: string[] = [];

function createNode(overrides: Partial<NodeConfig> = {}): NodeConfig {
  return {
    id: "node-private",
    name: "Private",
    type: "neo-cli",
    network: "private",
    syncMode: "full",
    version: "3.8.0",
    ports: { rpc: 20332, p2p: 20333, websocket: 20334, metrics: 22112 },
    paths: {
      base: "/tmp/node-private",
      data: "/tmp/node-private/data",
      logs: "/tmp/node-private/logs",
      config: "/tmp/node-private/config",
    },
    settings: {
      customConfig: { privateNetwork },
    },
    createdAt: 1,
    updatedAt: 1,
    ...overrides,
  };
}

describe("ConfigManager private network configuration", () => {
  beforeEach(() => {
    vi.mocked(DownloadManager.getNodeBinaryPath).mockReturnValue(null);
  });

  afterEach(() => {
    vi.clearAllMocks();
    for (const dir of tempDirs.splice(0)) {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });

  it("writes private network protocol settings into neo-cli config", async () => {
    const config = await ConfigManager.generateNeoCliConfig(createNode());

    expect(config.ProtocolConfiguration).toMatchObject({
      Network: 987654,
      ValidatorsCount: 4,
      StandbyCommittee: privateNetwork.standbyCommittee,
      SeedList: privateNetwork.seedList,
    });
  });

  it("writes private network protocol settings into neo-go config", () => {
    const config = ConfigManager.generateNeoGoConfig(createNode({ type: "neo-go" }));

    expect(config.ProtocolConfiguration).toMatchObject({
      Magic: 987654,
      ValidatorsCount: 4,
      StandbyCommittee: privateNetwork.standbyCommittee,
      SeedList: privateNetwork.seedList,
    });
  });

  it("does not let custom private network settings override mainnet or testnet", async () => {
    const neoCliConfig = await ConfigManager.generateNeoCliConfig(createNode({ network: "mainnet" }));
    const neoGoConfig = ConfigManager.generateNeoGoConfig(createNode({ type: "neo-go", network: "testnet" }));

    expect((neoCliConfig.ProtocolConfiguration as any).Network).not.toBe(987654);
    expect((neoGoConfig.ProtocolConfiguration as any).Magic).not.toBe(987654);
    expect((neoGoConfig.ProtocolConfiguration as any).SeedList).not.toEqual(privateNetwork.seedList);
  });

  it("does not inherit mainnet protocol settings for private neo-cli without custom private network config", async () => {
    const binaryDir = fs.mkdtempSync(path.join(os.tmpdir(), "neonexus-private-base-"));
    tempDirs.push(binaryDir);
    fs.writeFileSync(path.join(binaryDir, "config.mainnet.json"), JSON.stringify({
      ProtocolConfiguration: {
        Network: 860833102,
        ValidatorsCount: 21,
        StandbyCommittee: ["02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"],
        SeedList: ["seed1.neo.org:10333"],
        Hardforks: {
          Basilisk: 4120000,
        },
      },
      ApplicationConfiguration: {},
    }));
    vi.spyOn(DownloadManager, "getNodeBinaryPath").mockReturnValue(binaryDir);

    const config = await ConfigManager.generateNeoCliConfig(createNode({ settings: {} }));

    expect(config.ProtocolConfiguration).toMatchObject({
      Network: 56753,
      ValidatorsCount: 7,
      StandbyCommittee: [],
      SeedList: [],
    });
    expect(config.ProtocolConfiguration).not.toHaveProperty("Hardforks");
  });
});
