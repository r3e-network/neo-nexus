import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { NeofuraNode } from "../../src/nodes/NeofuraNode";
import type { NodeConfig } from "../../src/types/index";

function makeConfig(overrides?: Partial<NodeConfig>): NodeConfig {
  return {
    id: "node-test",
    name: "Test Neofura",
    chain: "n3",
    type: "neofura",
    network: "mainnet",
    syncMode: "full",
    version: "external",
    ports: { rpc: 0, p2p: 0 },
    paths: {
      base: "/tmp/neofura",
      data: "/tmp/neofura/data",
      logs: "/tmp/neofura/logs",
      config: "/tmp/neofura/config",
    },
    settings: {
      customConfig: { endpoint: "https://api.n3index.dev/mainnet" },
    },
    createdAt: 1,
    updatedAt: 1,
    ...overrides,
  };
}

describe("NeofuraNode", () => {
  let originalFetch: typeof fetch;

  beforeEach(() => {
    originalFetch = globalThis.fetch;
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
  });

  it("start()/stop() flips state without spawning a process", async () => {
    const node = new NeofuraNode(makeConfig());
    expect(node.getStatus().status).toBe("stopped");

    await node.start();
    expect(node.getStatus().status).toBe("running");
    expect(node.getStatus().pid).toBeUndefined();

    await node.stop();
    expect(node.getStatus().status).toBe("stopped");
    // External process is never killed; the lastStopped uptime
    // should reflect the in-memory observation window.
    expect(node.getStatus().uptime).toBeGreaterThanOrEqual(0);
  });

  it("getBlockHeight() reads last_indexed_block from /summary", async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ data: { last_indexed_block: 9_660_000 } }),
    }) as unknown as typeof fetch;

    const node = new NeofuraNode(makeConfig());
    const height = await node.getBlockHeight();
    expect(height).toBe(9_660_000);

    const calls = (globalThis.fetch as unknown as { mock: { calls: [string][] } }).mock.calls;
    expect(calls[0][0]).toBe("https://api.n3index.dev/mainnet/summary");
  });

  it("getBlockHeight() returns null when endpoint is missing", async () => {
    const node = new NeofuraNode(
      makeConfig({ settings: { customConfig: {} } }),
    );
    const height = await node.getBlockHeight();
    expect(height).toBeNull();
  });

  it("getBlockHeight() returns null on HTTP error (graceful degradation)", async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 503,
      json: async () => ({}),
    }) as unknown as typeof fetch;

    const node = new NeofuraNode(makeConfig());
    expect(await node.getBlockHeight()).toBeNull();
  });

  it("getPeersCount() always returns null (neofura has no peer concept)", async () => {
    const node = new NeofuraNode(makeConfig());
    expect(await node.getPeersCount()).toBeNull();
  });

  it("observe-only mode: getBinaryPath() throws with a clear message", async () => {
    const node = new NeofuraNode(makeConfig());
    await expect(node.getBinaryPath()).rejects.toThrow(/observe-only mode/);
  });

  it("managed mode: getBinaryPath() returns the configured path when the binary exists", async () => {
    // Use the running node executable as a stand-in real file —
    // every machine that runs vitest has it on disk.
    const realBinary = process.execPath;
    const node = new NeofuraNode(
      makeConfig({
        ports: { rpc: 8081, p2p: 0 },
        settings: {
          customConfig: { binaryPath: realBinary, workingDir: "/tmp/neofura" },
        },
      }),
    );
    await expect(node.getBinaryPath()).resolves.toBe(realBinary);
  });

  it("managed mode: getBinaryPath() throws when the configured binary is missing", async () => {
    const node = new NeofuraNode(
      makeConfig({
        settings: {
          customConfig: { binaryPath: "/nonexistent/neo3fura_http" },
        },
      }),
    );
    await expect(node.getBinaryPath()).rejects.toThrow(/not found/);
  });

  it("managed mode: getBlockHeight() falls back to localhost:rpc when no endpoint is configured", async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ data: { last_indexed_block: 42 } }),
    }) as unknown as typeof fetch;

    const node = new NeofuraNode(
      makeConfig({
        ports: { rpc: 8181, p2p: 0 },
        settings: {
          customConfig: { binaryPath: process.execPath },
        },
      }),
    );
    const height = await node.getBlockHeight();
    expect(height).toBe(42);

    const calls = (globalThis.fetch as unknown as { mock: { calls: [string][] } }).mock.calls;
    expect(calls[0][0]).toBe("http://127.0.0.1:8181/summary");
  });

  it("getWorkingDirectory() honors customConfig.workingDir when set", () => {
    const node = new NeofuraNode(
      makeConfig({
        settings: {
          customConfig: {
            binaryPath: process.execPath,
            workingDir: "/srv/neo3fura",
          },
        },
      }),
    );
    expect(node.getWorkingDirectory()).toBe("/srv/neo3fura");
  });
});
