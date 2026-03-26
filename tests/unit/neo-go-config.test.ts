import { describe, expect, it } from "vitest";
import { ConfigManager } from "../../src/core/ConfigManager";

describe("ConfigManager.generateNeoGoConfig", () => {
  it("emits the current neo-go config shape", () => {
    const config = ConfigManager.generateNeoGoConfig({
      id: "node-1",
      name: "Test",
      type: "neo-go",
      network: "mainnet",
      syncMode: "full",
      version: "v0.118.0",
      ports: { rpc: 10332, p2p: 10333, metrics: 2112 },
      paths: {
        base: "/tmp/node-1",
        data: "/tmp/node-1/data",
        logs: "/tmp/node-1/logs",
        config: "/tmp/node-1/config",
      },
      settings: {
        minPeers: 8,
        maxPeers: 50,
        relay: false,
      },
      createdAt: 1,
      updatedAt: 1,
    });

    expect(config).toMatchObject({
      ProtocolConfiguration: {
        Magic: 860833102,
        MaxBlockSystemFee: 2000000000,
        TimePerBlock: "15s",
        MemPoolSize: 50000,
        ValidatorsCount: 7,
        Hardforks: {
          Aspidochelone: 1730000,
          Basilisk: 4120000,
          Cockatrice: 5450000,
          Domovoi: 5570000,
          Echidna: 7300000,
          Faun: 8800000,
        },
      },
      ApplicationConfiguration: {
        SkipBlockVerification: false,
        DBConfiguration: {
          Type: "leveldb",
          LevelDBOptions: {
            DataDirectoryPath: "./data",
          },
        },
        P2P: {
          Addresses: [":10333"],
          MaxPeers: 50,
          AttemptConnPeers: 8,
          MinPeers: 8,
          PingInterval: "30s",
          PingTimeout: "90s",
        },
        Relay: false,
        Consensus: {
          Enabled: false,
        },
        Oracle: {
          Enabled: false,
        },
        P2PNotary: {
          Enabled: false,
        },
        RPC: {
          Enabled: true,
          Addresses: [":10332"],
          TLSConfig: {
            Enabled: false,
            CertFile: "serv.crt",
            KeyFile: "serv.key",
          },
        },
        Prometheus: {
          Enabled: true,
          Addresses: [":2112"],
        },
      },
    });

    expect((config.ProtocolConfiguration as any).StandbyCommittee[0]).toBe(
      "03b209fd4f53a7170ea4444e0cb0a6bb6a53c2bd016926989cf85f9b0fba17a70c",
    );
    expect((config.ProtocolConfiguration as any).StandbyCommittee).toHaveLength(21);

    expect(JSON.stringify(config)).not.toContain("AddressVersion");
    expect(JSON.stringify(config)).not.toContain("DataDirectoryPath\":\"Data\"");
  });
});
