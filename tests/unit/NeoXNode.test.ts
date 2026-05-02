import { describe, expect, it, vi } from "vitest";

vi.doUnmock("better-sqlite3");

const { NeoXNode } = await import("../../src/nodes/NeoXNode");
const { getNeoXAssetInfo } = await import("../../src/core/DownloadManager");

function makeXConfig(network: "neox-mainnet" | "neox-testnet" = "neox-mainnet") {
  return {
    id: "node-x1",
    name: "X-1",
    chain: "x" as const,
    type: "neox-go" as const,
    network,
    syncMode: "full" as const,
    version: "v0.5.3",
    ports: { rpc: 8551, p2p: 30303, websocket: 8571, metrics: 8561 },
    paths: { base: "/tmp/x1", data: "/tmp/x1/data", logs: "/tmp/x1/logs", config: "/tmp/x1/config" },
    settings: {},
    createdAt: 0,
    updatedAt: 0,
  };
}

describe("Neo X download asset selection", () => {
  it("returns the geth-linux-amd64 asset for x86_64 Linux", () => {
    const info = getNeoXAssetInfo("v0.5.3", "linux", "x64");
    expect(info.assetName).toBe("geth-linux-amd64");
    expect(info.binaryName).toBe("geth");
    expect(info.downloadUrl).toBe(
      "https://github.com/bane-labs/go-ethereum/releases/download/v0.5.3/geth-linux-amd64",
    );
  });

  it("returns the arm64 asset for arm64 Linux", () => {
    const info = getNeoXAssetInfo("v0.5.3", "linux", "arm64");
    expect(info.assetName).toBe("geth-linux-arm64");
  });

  it("rejects non-Linux platforms because bane-labs only ships Linux binaries", () => {
    expect(() => getNeoXAssetInfo("v0.5.3", "darwin", "arm64")).toThrow(/Linux only/i);
    expect(() => getNeoXAssetInfo("v0.5.3", "win32", "x64")).toThrow(/Linux only/i);
  });
});

describe("NeoXNode RPC adapters", () => {
  it("getStartArgs uses mainnet network id and bootnodes for neox-mainnet", () => {
    const node = new NeoXNode(makeXConfig("neox-mainnet"));
    const args = node.getStartArgs();
    expect(args).toContain("--networkid");
    expect(args[args.indexOf("--networkid") + 1]).toBe("47763");
    const bootIdx = args.indexOf("--bootnodes");
    expect(bootIdx).toBeGreaterThan(-1);
    expect(args[bootIdx + 1]).toContain("enode://");
  });

  it("getStartArgs uses the testnet network id for neox-testnet", () => {
    const node = new NeoXNode(makeXConfig("neox-testnet"));
    const args = node.getStartArgs();
    expect(args[args.indexOf("--networkid") + 1]).toBe("12227332");
  });

  it("parses eth_blockNumber hex into a decimal block height", async () => {
    const node = new NeoXNode(makeXConfig());
    // Stub executeRpc on the prototype because it's defined on BaseNode.
    const spy = vi.spyOn(node as unknown as { executeRpc: () => Promise<string> }, "executeRpc")
      .mockResolvedValue('"0x10ada"');
    const height = await node.getBlockHeight();
    expect(spy).toHaveBeenCalledWith("eth_blockNumber");
    expect(height).toBe(0x10ada);
  });

  it("returns null when eth_blockNumber returns a non-hex payload", async () => {
    const node = new NeoXNode(makeXConfig());
    vi.spyOn(node as unknown as { executeRpc: () => Promise<string> }, "executeRpc")
      .mockResolvedValue('null');
    expect(await node.getBlockHeight()).toBeNull();
  });

  it("parses net_peerCount and eth_chainId", async () => {
    const node = new NeoXNode(makeXConfig("neox-mainnet"));
    const spy = vi.spyOn(
      node as unknown as { executeRpc: (method: string) => Promise<string> },
      "executeRpc",
    ).mockImplementation(async (method: string) => {
      if (method === "net_peerCount") return '"0x5"';
      if (method === "eth_chainId") return '"0xbab3"';
      return '""';
    });
    expect(await node.getPeersCount()).toBe(5);
    expect(await node.getChainId()).toBe(0xbab3);
    expect(spy).toHaveBeenCalledWith("net_peerCount");
    expect(spy).toHaveBeenCalledWith("eth_chainId");
  });
});
