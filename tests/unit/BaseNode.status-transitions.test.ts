import { EventEmitter } from "node:events";
import { beforeEach, describe, expect, it, vi } from "vitest";

const hoisted = vi.hoisted(() => ({
  spawnProcess: vi.fn(),
  killProcess: vi.fn(async () => true),
  getProcessInfo: vi.fn(async () => null),
}));

vi.mock("../../src/utils/exec", () => ({
  spawnProcess: hoisted.spawnProcess,
  killProcess: hoisted.killProcess,
  getProcessInfo: hoisted.getProcessInfo,
}));

describe("BaseNode status transitions", () => {
  beforeEach(() => {
    vi.resetModules();
    hoisted.spawnProcess.mockReset();
  });

  it("emits the correct previous status when startup begins", async () => {
    const { BaseNode } = await import("../../src/nodes/BaseNode");

    class TestNode extends BaseNode {
      async getBinaryPath(): Promise<string> {
        return "neo";
      }
      getStartArgs(): string[] {
        return [];
      }
      getWorkingDirectory(): string {
        return "/tmp";
      }
      async getBlockHeight(): Promise<number | null> {
        return null;
      }
      async getPeersCount(): Promise<number | null> {
        return null;
      }
      protected delay(): Promise<void> {
        return Promise.resolve();
      }
    }

    const child = new EventEmitter() as EventEmitter & {
      pid: number;
      exitCode: number | null;
      stdout: EventEmitter;
      stderr: EventEmitter;
    };
    child.pid = 12345;
    child.exitCode = null;
    child.stdout = new EventEmitter();
    child.stderr = new EventEmitter();
    hoisted.spawnProcess.mockReturnValue(child);

    const node = new TestNode({
      id: "node-1",
      name: "Node 1",
      type: "neo-go",
      network: "mainnet",
      syncMode: "full",
      version: "0.106.0",
      ports: { rpc: 10332, p2p: 10333 },
      paths: { base: "/tmp/node-1", data: "/tmp/node-1/data", logs: "/tmp/node-1/logs", config: "/tmp/node-1/config" },
      settings: {},
      createdAt: 1,
      updatedAt: 1,
    });

    const statuses: Array<{ status: string; previous: string }> = [];
    node.on("status", (status, previous) => {
      statuses.push({ status, previous });
    });

    await node.start();

    expect(statuses[0]).toEqual({
      status: "starting",
      previous: "stopped",
    });
  });

  it("emits the correct previous status when a running node errors", async () => {
    const { BaseNode } = await import("../../src/nodes/BaseNode");

    class TestNode extends BaseNode {
      async getBinaryPath(): Promise<string> {
        return "neo";
      }
      getStartArgs(): string[] {
        return [];
      }
      getWorkingDirectory(): string {
        return "/tmp";
      }
      async getBlockHeight(): Promise<number | null> {
        return null;
      }
      async getPeersCount(): Promise<number | null> {
        return null;
      }
      protected delay(): Promise<void> {
        return Promise.resolve();
      }
    }

    const child = new EventEmitter() as EventEmitter & {
      pid: number;
      exitCode: number | null;
      stdout: EventEmitter;
      stderr: EventEmitter;
    };
    child.pid = 12345;
    child.exitCode = null;
    child.stdout = new EventEmitter();
    child.stderr = new EventEmitter();
    hoisted.spawnProcess.mockReturnValue(child);

    const node = new TestNode({
      id: "node-1",
      name: "Node 1",
      type: "neo-go",
      network: "mainnet",
      syncMode: "full",
      version: "0.106.0",
      ports: { rpc: 10332, p2p: 10333 },
      paths: { base: "/tmp/node-1", data: "/tmp/node-1/data", logs: "/tmp/node-1/logs", config: "/tmp/node-1/config" },
      settings: {},
      createdAt: 1,
      updatedAt: 1,
    });

    const statuses: Array<{ status: string; previous: string }> = [];
    node.on("status", (status, previous) => {
      statuses.push({ status, previous });
    });
    node.on("error", () => {});

    await node.start();
    child.emit("error", new Error("boom"));

    expect(statuses.at(-1)).toEqual({
      status: "error",
      previous: "running",
    });
  });
});
