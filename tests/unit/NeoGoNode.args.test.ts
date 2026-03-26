import { describe, expect, it } from "vitest";
import { NeoGoNode } from "../../src/nodes/NeoGoNode";

describe("NeoGoNode getStartArgs", () => {
  it("includes a value for --relative-path", () => {
    const node = new NeoGoNode({
      id: "node-1",
      name: "Test",
      type: "neo-go",
      network: "mainnet",
      syncMode: "full",
      version: "v0.118.0",
      ports: { rpc: 10332, p2p: 10333 },
      paths: {
        base: "/tmp/node-1",
        data: "/tmp/node-1/data",
        logs: "/tmp/node-1/logs",
        config: "/tmp/node-1/config",
      },
      settings: {},
      createdAt: 1,
      updatedAt: 1,
    });

    expect(node.getStartArgs()).toEqual([
      "node",
      "--config-file",
      "/tmp/node-1/config/protocol.yml",
      "--relative-path",
      "/tmp/node-1",
    ]);
  });
});
