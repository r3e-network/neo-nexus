import { readFileSync, rmSync } from "node:fs";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("node:fs", async (importOriginal) => {
  const actual = await importOriginal<typeof import("node:fs")>();
  return {
    ...actual,
    rmSync: vi.fn(),
  };
});

vi.mock("node:child_process", async (importOriginal) => {
  const actual = await importOriginal<typeof import("node:child_process")>();
  return {
    ...actual,
    execFileSync: vi.fn(),
  };
});

vi.mock("../../src/utils/lifecycle", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../../src/utils/lifecycle")>();
  return {
    ...actual,
    isProcessAlive: vi.fn(),
    getProcessCommand: vi.fn(),
    getProcessCwd: vi.fn(),
    getProcessArgv: vi.fn(),
  };
});

import { ConfigManager } from "../../src/core/ConfigManager";
import { DownloadManager } from "../../src/core/DownloadManager";
import { NodeManager } from "../../src/core/NodeManager";
import { StorageManager } from "../../src/core/StorageManager";
import { NeoCliNode } from "../../src/nodes/NeoCliNode";
import * as lifecycle from "../../src/utils/lifecycle";
import { getNodePath, paths } from "../../src/utils/paths";
import { execFileSync } from "node:child_process";

const pkg = JSON.parse(
  readFileSync(new URL("../../package.json", import.meta.url), "utf-8"),
) as { version: string };

describe("NodeManager system actions", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.mocked(lifecycle.isProcessAlive).mockReset();
    vi.mocked(lifecycle.getProcessCommand).mockReset();
    vi.mocked(lifecycle.getProcessCwd).mockReset();
    vi.mocked(lifecycle.getProcessArgv).mockReset();
    vi.mocked(execFileSync).mockReset();
    vi.mocked(rmSync).mockReset();
  });

  it("stops only the nodes that are currently running", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getAllNodes: ReturnType<typeof vi.fn>;
      stopNode: ReturnType<typeof vi.fn>;
    };

    manager.getAllNodes = vi.fn(() => [
      { id: "node-1", process: { status: "running" } },
      { id: "node-2", process: { status: "stopped" } },
      { id: "node-3", process: { status: "running" } },
    ]);
    manager.stopNode = vi.fn().mockResolvedValue(undefined);

    const result = await manager.stopAllNodes();

    expect(result).toEqual({
      stoppedCount: 2,
      alreadyStoppedCount: 1,
    });
    expect(manager.stopNode).toHaveBeenCalledTimes(2);
    expect(manager.stopNode).toHaveBeenCalledWith("node-1");
    expect(manager.stopNode).toHaveBeenCalledWith("node-3");
  });

  it("does not abort stop-all when observe-only imported running nodes reject process control", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getAllNodes: ReturnType<typeof vi.fn>;
      stopNode: ReturnType<typeof vi.fn>;
    };

    manager.getAllNodes = vi.fn(() => [
      { id: "managed", process: { status: "running" } },
      { id: "observed", process: { status: "running" }, settings: { import: { imported: true, ownershipMode: "observe-only" } } },
    ]);
    manager.stopNode = vi
      .fn()
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(Object.assign(new Error("denied"), { code: "NODE_OWNERSHIP_DENIED" }));

    await expect(manager.stopAllNodes()).resolves.toEqual({ stoppedCount: 1, alreadyStoppedCount: 1 });
  });

  it("aggregates cleaned log files across all nodes", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getAllNodes: ReturnType<typeof vi.fn>;
    };

    manager.getAllNodes = vi.fn(() => [
      { id: "node-1", paths: { logs: "/tmp/node-1/logs" } },
      { id: "node-2", paths: { logs: "/tmp/node-2/logs" } },
    ]);

    vi.spyOn(StorageManager, "cleanOldLogs")
      .mockResolvedValueOnce(5)
      .mockResolvedValueOnce(0);

    const result = await manager.cleanOldLogs(14);

    expect(result).toEqual({
      cleanedFiles: 5,
      nodesAffected: 1,
      maxAgeDays: 14,
    });
  });

  it("skips observe-only imported node logs during cleanup", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getAllNodes: ReturnType<typeof vi.fn>;
    };

    manager.getAllNodes = vi.fn(() => [
      { id: "managed", paths: { logs: "/tmp/managed/logs" }, settings: {} },
      { id: "observed", paths: { logs: "/opt/neo/Logs" }, settings: { import: { imported: true, ownershipMode: "observe-only" } } },
    ]);

    vi.spyOn(StorageManager, "cleanOldLogs").mockResolvedValueOnce(3);

    const result = await manager.cleanOldLogs(7);

    expect(StorageManager.cleanOldLogs).toHaveBeenCalledTimes(1);
    expect(StorageManager.cleanOldLogs).toHaveBeenCalledWith("/tmp/managed/logs", 7);
    expect(result).toEqual({ cleanedFiles: 3, nodesAffected: 1, maxAgeDays: 7 });
  });

  it("blocks direct log cleanup for observe-only imported nodes", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.getNode = vi.fn(() => ({
      id: "observed",
      paths: { logs: "/opt/neo/Logs" },
      settings: { import: { imported: true, ownershipMode: "observe-only" } },
    }));
    vi.spyOn(StorageManager, "cleanOldLogs").mockResolvedValue(3);

    await expect(manager.cleanNodeLogs("observed", 7)).rejects.toMatchObject({ code: "NODE_OWNERSHIP_DENIED" });
    expect(StorageManager.cleanOldLogs).not.toHaveBeenCalled();
  });

  it("exports a configuration snapshot without process metrics noise", () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getAllNodes: ReturnType<typeof vi.fn>;
    };

    manager.getAllNodes = vi.fn(() => [
      {
        id: "node-1",
        name: "Mainnet Node",
        type: "neo-cli",
        network: "mainnet",
        version: "3.9.2",
        syncMode: "full",
        ports: { rpc: 10332, p2p: 10333 },
        paths: {
          base: "/nodes/node-1",
          data: "/nodes/node-1/data",
          logs: "/nodes/node-1/logs",
          config: "/nodes/node-1/config",
        },
        settings: { Example: true },
        createdAt: 1,
        updatedAt: 2,
        process: { status: "running", pid: 42 },
        metrics: { blockHeight: 1000 },
        plugins: [{ id: "RpcServer", enabled: true }],
      },
    ]);

    const snapshot = manager.exportConfiguration();

    expect(snapshot.version).toBe(pkg.version);
    expect(snapshot.version).not.toBe("2.0.0");
    expect(snapshot.nodes).toEqual([
      {
        id: "node-1",
        name: "Mainnet Node",
        type: "neo-cli",
        network: "mainnet",
        version: "3.9.2",
        syncMode: "full",
        ports: { rpc: 10332, p2p: 10333 },
        paths: {
          base: "/nodes/node-1",
          data: "/nodes/node-1/data",
          logs: "/nodes/node-1/logs",
          config: "/nodes/node-1/config",
        },
        settings: { Example: true },
        createdAt: 1,
        updatedAt: 2,
        plugins: [{ id: "RpcServer", enabled: true }],
      },
    ]);
    expect(snapshot.nodes[0]).not.toHaveProperty("process");
    expect(snapshot.nodes[0]).not.toHaveProperty("metrics");
  });

  it("refuses to stop an in-memory imported process after ownership is downgraded", async () => {
    const nodeInstance = {
      stop: vi.fn().mockResolvedValue(undefined),
      isRunning: vi.fn(() => true),
    };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      repo: { updateStatus: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map([["node-1", nodeInstance]]);
    manager.repo = { updateStatus: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      process: { status: "running", pid: 4321 },
      settings: { import: { imported: true, ownershipMode: "observe-only" } },
    }));

    await expect(manager.stopNode("node-1")).rejects.toMatchObject({ code: "NODE_OWNERSHIP_DENIED" });
    expect(nodeInstance.stop).not.toHaveBeenCalled();
    expect(manager.repo.updateStatus).not.toHaveBeenCalledWith("node-1", "stopped");
  });

  it("treats legacy external nodes without import metadata as observe-only", async () => {
    const nodeInstance = {
      stop: vi.fn().mockResolvedValue(undefined),
      isRunning: vi.fn(() => true),
    };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      repo: { updateStatus: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map([["node-legacy", nodeInstance]]);
    manager.repo = { updateStatus: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-legacy",
      type: "neo-cli",
      paths: { base: "/opt/legacy-neo", config: "/opt/legacy-neo/config.json", data: "/opt/legacy-neo/Chain", logs: "/opt/legacy-neo/Logs" },
      settings: {},
      process: { status: "running", pid: 4321 },
    }));

    await expect(manager.stopNode("node-legacy")).rejects.toMatchObject({ code: "NODE_OWNERSHIP_DENIED" });
    expect(nodeInstance.stop).not.toHaveBeenCalled();
    expect(manager.repo.updateStatus).not.toHaveBeenCalledWith("node-legacy", "stopped");
  });

  it("strips reserved import metadata from newly created managed nodes", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      portManager: { findNextIndex: ReturnType<typeof vi.fn>; allocatePorts: ReturnType<typeof vi.fn> };
      repo: { transaction: (fn: () => void) => void; saveNode: ReturnType<typeof vi.fn>; deleteNode: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };
    let savedConfig: { settings?: unknown } | undefined;

    manager.portManager = {
      findNextIndex: vi.fn().mockResolvedValue(1),
      allocatePorts: vi.fn().mockResolvedValue({ rpc: 20332, p2p: 20333, websocket: 20334, metrics: 20335 }),
    };
    manager.repo = {
      transaction: (fn: () => void) => fn(),
      saveNode: vi.fn((config) => { savedConfig = config; }),
      deleteNode: vi.fn(),
    };
    manager.getNode = vi.fn(() => savedConfig as never);
    vi.spyOn(DownloadManager, "hasNodeBinary").mockReturnValue(true);
    vi.spyOn(StorageManager, "ensureNodeDirectories").mockImplementation(() => undefined);
    vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    const node = await manager.createNode({
      name: "Managed",
      type: "neo-cli",
      network: "testnet",
      version: "3.9.2",
      settings: {
        debugMode: true,
        import: { imported: true, ownershipMode: "observe-only", existingPath: "/opt/neo", importedAt: 1 },
      },
    });

    expect(node.settings).toEqual({ debugMode: true });
    expect(manager.repo.saveNode).toHaveBeenCalledOnce();
  });

  it("strips reserved import metadata while restoring managed snapshots", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      createNode: ReturnType<typeof vi.fn>;
      installPlugin: ReturnType<typeof vi.fn>;
    };

    manager.createNode = vi.fn().mockResolvedValue({ id: "new-node-1", type: "neo-cli" });
    manager.installPlugin = vi.fn().mockResolvedValue(undefined);

    await manager.restoreConfiguration({
      version: "2.0.0",
      nodes: [{
        name: "Restored",
        type: "neo-cli",
        network: "testnet",
        syncMode: "full",
        version: "3.9.2",
        ports: { rpc: 20332, p2p: 20333 },
        settings: {
          debugMode: true,
          import: { imported: true, ownershipMode: "observe-only", existingPath: "/opt/neo", importedAt: 1 },
        },
      }],
    });

    expect(manager.createNode).toHaveBeenCalledWith(expect.objectContaining({
      settings: { debugMode: true },
    }));
  });

  it("does not start a node when the repository PID still matches the imported native process", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      paths: { base: "/opt/neo-cli", config: "/opt/neo-cli/config.json", data: "/opt/neo-cli/Chain", logs: "/opt/neo-cli/Logs" },
      settings: { import: { imported: true, ownershipMode: "managed-process", existingPath: "/opt/neo-cli", importedAt: 1 } },
      process: { status: "running", pid: 4321 },
    }));
    vi.mocked(lifecycle.isProcessAlive).mockReturnValue(true);
    vi.mocked(lifecycle.getProcessCommand).mockReturnValue("dotnet /opt/neo-cli/neo-cli.dll");
    vi.mocked(lifecycle.getProcessCwd).mockReturnValue("/opt/neo-cli");

    await expect(manager.startNode("node-1")).rejects.toMatchObject({
      code: "NODE_ALREADY_RUNNING",
    });
  });

  it("reconciles a stale repository PID before starting the node", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      repo: { updateStatus: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.repo = { updateStatus: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      name: "Imported",
      type: "neo-cli",
      network: "testnet",
      syncMode: "full",
      version: "3.8.0",
      ports: { rpc: 20332, p2p: 20333 },
      paths: { base: "/opt/neo-cli", config: "/opt/neo-cli/config.json", data: "/opt/neo-cli/Chain", logs: "/opt/neo-cli/Logs" },
      settings: { import: { imported: true, ownershipMode: "managed-process", existingPath: "/opt/neo-cli", importedAt: 1 } },
      createdAt: 1,
      updatedAt: 1,
      process: { status: "running", pid: 4321 },
    }));
    vi.mocked(lifecycle.isProcessAlive).mockReturnValue(false);
    const startSpy = vi.spyOn(NeoCliNode.prototype, "start").mockResolvedValue(undefined);
    vi.spyOn(NeoCliNode.prototype, "getStatus").mockReturnValue({ status: "running", pid: 9876 });

    await manager.startNode("node-1");

    expect(startSpy).toHaveBeenCalledOnce();
    expect(manager.repo.updateStatus).toHaveBeenNthCalledWith(1, "node-1", "stopped");
    expect(manager.repo.updateStatus).toHaveBeenLastCalledWith("node-1", "running", 9876);
  });

  it("rejects managed-process imported neo-cli start when the adopted config is not base config.json", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.getNode = vi.fn(() => ({
      id: "node-imported",
      name: "Imported TestNet",
      type: "neo-cli",
      network: "testnet",
      syncMode: "full",
      version: "3.8.0",
      ports: { rpc: 20332, p2p: 20333 },
      paths: { base: "/opt/neo-cli", config: "/opt/neo-cli/config.testnet.json", data: "/opt/neo-cli/Chain", logs: "/opt/neo-cli/Logs" },
      settings: { import: { imported: true, ownershipMode: "managed-process", existingPath: "/opt/neo-cli", importedAt: 1 } },
      createdAt: 1,
      updatedAt: 1,
      process: { status: "stopped" },
    }));
    const startSpy = vi.spyOn(NeoCliNode.prototype, "start").mockResolvedValue(undefined);

    await expect(manager.startNode("node-imported")).rejects.toMatchObject({
      code: "IMPORTED_NEO_CLI_CONFIG_START_UNSUPPORTED",
    });
    expect(startSpy).not.toHaveBeenCalled();
  });

  it("stops a repository-attached running process only after validating the PID belongs to the node", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      repo: { updateStatus: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.repo = { updateStatus: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      paths: { base: "/opt/neo-cli", config: "/opt/neo-cli/config.json", data: "/opt/neo-cli/Chain", logs: "/opt/neo-cli/Logs" },
      settings: { import: { imported: true, ownershipMode: "managed-process", existingPath: "/opt/neo-cli", importedAt: 1 } },
      process: { status: "running", pid: 4321 },
    }));
    vi.mocked(lifecycle.isProcessAlive)
      .mockReturnValueOnce(true)
      .mockReturnValueOnce(false);
    vi.mocked(lifecycle.getProcessCommand).mockReturnValue("dotnet /opt/neo-cli/neo-cli.dll");
    vi.mocked(lifecycle.getProcessCwd).mockReturnValue("/opt/neo-cli");
    const killSpy = vi.spyOn(process, "kill").mockImplementation(() => true as never);

    await manager.stopNode("node-1");

    expect(killSpy).toHaveBeenCalledWith(4321, "SIGTERM");
    expect(manager.repo.updateStatus).toHaveBeenCalledWith("node-1", "stopping", 4321);
    expect(manager.repo.updateStatus).toHaveBeenCalledWith("node-1", "stopped");
  });

  it("does not kill an untrusted repository PID and reconciles the node as stopped", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      repo: { updateStatus: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.repo = { updateStatus: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-go",
      paths: { base: "/opt/neo-go-node", config: "/opt/neo-go-node/protocol.yml", data: "/opt/neo-go-node/data", logs: "/opt/neo-go-node/logs" },
      settings: { import: { imported: true, ownershipMode: "managed-process", existingPath: "/opt/neo-go-node", importedAt: 1 } },
      process: { status: "running", pid: 4321 },
    }));
    vi.mocked(lifecycle.isProcessAlive).mockReturnValue(true);
    vi.mocked(lifecycle.getProcessCommand).mockReturnValue("/usr/bin/python unrelated.py");
    vi.mocked(lifecycle.getProcessCwd).mockReturnValue("/tmp");
    const killSpy = vi.spyOn(process, "kill").mockImplementation(() => true as never);

    await manager.stopNode("node-1");

    expect(killSpy).not.toHaveBeenCalled();
    expect(manager.repo.updateStatus).toHaveBeenCalledWith("node-1", "stopped");
  });

  it("does not kill PIDs whose command path only shares the node path prefix", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      repo: { updateStatus: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.repo = { updateStatus: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      paths: { base: "/opt/neo", config: "/opt/neo/config.json", data: "/opt/neo/Chain", logs: "/opt/neo/Logs" },
      settings: { import: { imported: true, ownershipMode: "managed-process", existingPath: "/opt/neo", importedAt: 1 } },
      process: { status: "running", pid: 4321 },
    }));
    vi.mocked(lifecycle.isProcessAlive).mockReturnValue(true);
    vi.mocked(lifecycle.getProcessCommand).mockReturnValue("dotnet /opt/neo-other/neo-cli.dll");
    vi.mocked(lifecycle.getProcessCwd).mockReturnValue("/tmp");
    vi.mocked(lifecycle.getProcessArgv).mockReturnValue(["dotnet", "/opt/neo-other/neo-cli.dll"]);
    const killSpy = vi.spyOn(process, "kill").mockImplementation(() => true as never);

    await manager.stopNode("node-1");

    expect(killSpy).not.toHaveBeenCalled();
    expect(manager.repo.updateStatus).toHaveBeenCalledWith("node-1", "stopped");
  });

  it("preserves the attached PID when stop fails before the process exits", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      repo: { updateStatus: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.repo = { updateStatus: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      paths: { base: "/opt/neo-cli", config: "/opt/neo-cli/config.json", data: "/opt/neo-cli/Chain", logs: "/opt/neo-cli/Logs" },
      settings: { import: { imported: true, ownershipMode: "managed-process", existingPath: "/opt/neo-cli", importedAt: 1 } },
      process: { status: "running", pid: 4321 },
    }));
    vi.mocked(lifecycle.isProcessAlive).mockReturnValue(true);
    vi.mocked(lifecycle.getProcessCommand).mockReturnValue("dotnet /opt/neo-cli/neo-cli.dll");
    vi.mocked(lifecycle.getProcessCwd).mockReturnValue("/opt/neo-cli");
    vi.mocked(lifecycle.getProcessArgv).mockReturnValue(["dotnet", "/opt/neo-cli/neo-cli.dll"]);
    const error = Object.assign(new Error("permission denied"), { code: "EPERM" });
    vi.spyOn(process, "kill").mockImplementation(() => { throw error; });

    await expect(manager.stopNode("node-1")).rejects.toThrow("permission denied");

    expect(manager.repo.updateStatus).toHaveBeenCalledWith("node-1", "stopping", 4321);
    expect(manager.repo.updateStatus).toHaveBeenCalledWith("node-1", "error", 4321);
  });

  it("does not kill invalid repository PIDs", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      repo: { updateStatus: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.repo = { updateStatus: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      paths: { base: "/opt/neo-cli", config: "/opt/neo-cli/config.json", data: "/opt/neo-cli/Chain", logs: "/opt/neo-cli/Logs" },
      settings: { import: { imported: true, ownershipMode: "managed-process", existingPath: "/opt/neo-cli", importedAt: 1 } },
      process: { status: "running", pid: -4321 },
    }));
    const killSpy = vi.spyOn(process, "kill").mockImplementation(() => true as never);

    await manager.stopNode("node-1");

    expect(killSpy).not.toHaveBeenCalled();
    expect(manager.repo.updateStatus).toHaveBeenCalledWith("node-1", "stopped");
  });

  it("refuses to stop observe-only imported nodes even when the stored PID is trusted", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      repo: { updateStatus: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.repo = { updateStatus: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      paths: { base: "/opt/neo-cli", config: "/opt/neo-cli/config.json", data: "/opt/neo-cli/Chain", logs: "/opt/neo-cli/Logs" },
      process: { status: "running", pid: 4321 },
      settings: { import: { imported: true, ownershipMode: "observe-only" } },
    }));
    vi.mocked(lifecycle.isProcessAlive).mockReturnValue(true);
    vi.mocked(lifecycle.getProcessCommand).mockReturnValue("dotnet /opt/neo-cli/neo-cli.dll");
    vi.mocked(lifecycle.getProcessCwd).mockReturnValue("/opt/neo-cli");
    const killSpy = vi.spyOn(process, "kill").mockImplementation(() => true as never);

    await expect(manager.stopNode("node-1")).rejects.toMatchObject({ code: "NODE_OWNERSHIP_DENIED" });

    expect(killSpy).not.toHaveBeenCalled();
    expect(manager.repo.updateStatus).not.toHaveBeenCalledWith("node-1", "stopping", 4321);
  });

  it("blocks config writes for observe-only imported nodes", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.getNode = vi.fn(() => ({
      id: "node-1",
      name: "Imported",
      type: "neo-cli",
      process: { status: "stopped" },
      settings: { import: { imported: true, ownershipMode: "observe-only" } },
    }));

    await expect(manager.updateNode("node-1", { name: "Renamed" })).rejects.toMatchObject({
      code: "NODE_OWNERSHIP_DENIED",
    });
  });

  it("auto-attaches to the best matching candidate PID instead of the first pgrep result", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      repo: { updateStatus: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.repo = { updateStatus: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      ports: { rpc: 20332, p2p: 20333 },
      paths: { base: "/opt/neo", config: "/opt/neo/config.testnet.json", data: "/opt/neo/Chain", logs: "/opt/neo/Logs" },
      process: { status: "stopped" },
      settings: { import: { imported: true, ownershipMode: "managed-process" } },
    }));
    vi.mocked(execFileSync).mockReturnValue("111\n222\n" as never);
    vi.mocked(lifecycle.isProcessAlive).mockReturnValue(true);
    vi.mocked(lifecycle.getProcessCommand).mockImplementation((pid) =>
      pid === 111 ? "dotnet /opt/neo-other/neo-cli.dll" : "dotnet /opt/neo/neo-cli.dll --config-path /opt/neo/config.testnet.json",
    );
    vi.mocked(lifecycle.getProcessCwd).mockImplementation((pid) =>
      pid === 111 ? "/opt/neo-other" : "/opt/neo",
    );
    vi.mocked(lifecycle.getProcessArgv).mockImplementation((pid) =>
      pid === 111
        ? ["dotnet", "/opt/neo-other/neo-cli.dll"]
        : ["dotnet", "/opt/neo/neo-cli.dll", "--config-path", "/opt/neo/config.testnet.json", "--rpc-port", "20332"],
    );

    await (manager as unknown as { attachToRunningProcess: (nodeId: string, type: string) => Promise<void> })
      .attachToRunningProcess("node-1", "neo-cli");

    expect(manager.repo.updateStatus).toHaveBeenCalledWith("node-1", "running", 222);
  });

  it("treats ownership metadata as immutable during settings updates", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      repo: { updateNode: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    const importedSettings = {
      import: { imported: true, ownershipMode: "managed-config", existingPath: "/opt/neo", importedAt: 1 },
      debugMode: false,
    };
    manager.repo = { updateNode: vi.fn() };
    manager.getNode = vi
      .fn()
      .mockReturnValueOnce({
        id: "node-1",
        process: { status: "stopped" },
        settings: importedSettings,
        plugins: [],
      })
      .mockReturnValueOnce({
        id: "node-1",
        process: { status: "stopped" },
        settings: importedSettings,
        plugins: [],
      });
    vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await manager.updateNode("node-1", {
      settings: {
        debugMode: true,
        import: { imported: false, ownershipMode: "managed-process" },
      },
    });

    const savedSettings = JSON.parse(manager.repo.updateNode.mock.calls[0][2][0] as string);
    expect(savedSettings.import).toEqual(importedSettings.import);
    expect(savedSettings.debugMode).toBe(true);
  });

  it("rejects incompatible secure signer policy updates before persisting settings", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      repo: { updateNode: ReturnType<typeof vi.fn> };
      secureSignerManager: { getProfile: ReturnType<typeof vi.fn>; buildSignClientConfig: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.repo = { updateNode: vi.fn() };
    manager.secureSignerManager = {
      getProfile: vi.fn(() => ({ id: "signer-1", name: "Software", mode: "software", endpoint: "tcp://127.0.0.1:30333", enabled: true })),
      buildSignClientConfig: vi.fn(() => {
        throw new Error("Secure signer policy requires hardware-backed protection");
      }),
    };
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      process: { status: "stopped" },
      settings: {},
      plugins: [],
    }));
    vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

    await expect(manager.updateNode("node-1", {
      settings: {
        keyProtection: {
          mode: "secure-signer",
          signerProfileId: "signer-1",
          policy: { requireHardwareProtection: true },
        },
      },
    })).rejects.toThrow("hardware-backed protection");

    expect(manager.repo.updateNode).not.toHaveBeenCalled();
    expect(ConfigManager.writeNodeConfig).not.toHaveBeenCalled();
  });

  it("does not recursively remove node directories that only share the managed nodes path prefix", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      portManager: { releasePorts: ReturnType<typeof vi.fn> };
      repo: { deleteNode: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.portManager = { releasePorts: vi.fn() };
    manager.repo = { deleteNode: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-prefix",
      ports: { rpc: 20332, p2p: 20333 },
      paths: { base: `${paths.nodes}-other/node-prefix` },
      process: { status: "stopped" },
      settings: {},
    }));

    await manager.deleteNode("node-prefix", true);

    expect(rmSync).not.toHaveBeenCalled();
    expect(manager.repo.deleteNode).toHaveBeenCalledWith("node-prefix");
  });

  it("does not recursively remove imported node directories even when they are under the managed nodes directory", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      portManager: { releasePorts: ReturnType<typeof vi.fn> };
      repo: { deleteNode: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.portManager = { releasePorts: vi.fn() };
    manager.repo = { deleteNode: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "node-imported",
      ports: { rpc: 20332, p2p: 20333 },
      paths: { base: `${paths.nodes}/imported-node` },
      process: { status: "stopped" },
      settings: { import: { imported: true, ownershipMode: "managed-config" } },
    }));

    await manager.deleteNode("node-imported", true);

    expect(rmSync).not.toHaveBeenCalled();
    expect(manager.repo.deleteNode).toHaveBeenCalledWith("node-imported");
  });

  it("detaches but does not destroy an in-memory observe-only imported node during delete", async () => {
    const runningNode = {
      destroy: vi.fn(),
      detach: vi.fn(),
    };
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      portManager: { releasePorts: ReturnType<typeof vi.fn> };
      repo: { deleteNode: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
      stopNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map([["node-imported", runningNode]]);
    manager.portManager = { releasePorts: vi.fn() };
    manager.repo = { deleteNode: vi.fn() };
    manager.stopNode = vi.fn().mockRejectedValue(Object.assign(new Error("denied"), { code: "NODE_OWNERSHIP_DENIED" }));
    manager.getNode = vi.fn(() => ({
      id: "node-imported",
      ports: { rpc: 20332, p2p: 20333 },
      paths: { base: "/opt/external-neo" },
      process: { status: "running", pid: 4321 },
      settings: { import: { imported: true, ownershipMode: "observe-only", existingPath: "/opt/external-neo", importedAt: 1 } },
    }));

    await manager.deleteNode("node-imported", true);

    expect(manager.stopNode).toHaveBeenCalledWith("node-imported");
    expect(runningNode.destroy).not.toHaveBeenCalled();
    expect(runningNode.detach).toHaveBeenCalledOnce();
    expect(manager.nodes.has("node-imported")).toBe(false);
    expect(manager.repo.deleteNode).toHaveBeenCalledWith("node-imported");
    expect(rmSync).not.toHaveBeenCalled();
  });

  it("does not recursively remove malformed legacy node ids that resolve outside the managed nodes root", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      nodes: Map<string, unknown>;
      portManager: { releasePorts: ReturnType<typeof vi.fn> };
      repo: { deleteNode: ReturnType<typeof vi.fn> };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.nodes = new Map();
    manager.portManager = { releasePorts: vi.fn() };
    manager.repo = { deleteNode: vi.fn() };
    manager.getNode = vi.fn(() => ({
      id: "../logs",
      ports: { rpc: 20332, p2p: 20333 },
      paths: { base: `${paths.nodes}/../logs` },
      process: { status: "stopped" },
      settings: {},
    }));

    await manager.deleteNode("../logs", true);

    expect(rmSync).not.toHaveBeenCalled();
    expect(manager.repo.deleteNode).toHaveBeenCalledWith("../logs");
  });

  it("blocks plugin mutations for observe-only imported nodes", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      pluginManager: {
        installPlugin: ReturnType<typeof vi.fn>;
        updatePluginConfig: ReturnType<typeof vi.fn>;
        uninstallPlugin: ReturnType<typeof vi.fn>;
        setPluginEnabled: ReturnType<typeof vi.fn>;
        getInstalledPlugins: ReturnType<typeof vi.fn>;
      };
      getNode: ReturnType<typeof vi.fn>;
    };

    manager.pluginManager = {
      installPlugin: vi.fn(),
      updatePluginConfig: vi.fn(),
      uninstallPlugin: vi.fn(),
      setPluginEnabled: vi.fn(),
      getInstalledPlugins: vi.fn(() => []),
    };
    manager.getNode = vi.fn(() => ({
      id: "node-1",
      type: "neo-cli",
      version: "3.8.0",
      process: { status: "stopped" },
      settings: { import: { imported: true, ownershipMode: "observe-only" } },
    }));

    await expect(manager.installPlugin("node-1", "RpcServer", {})).rejects.toMatchObject({ code: "NODE_OWNERSHIP_DENIED" });
    expect(() => manager.updatePluginConfig("node-1", "RpcServer", {})).toThrow(/observe-only/);
    await expect(manager.uninstallPlugin("node-1", "RpcServer")).rejects.toMatchObject({ code: "NODE_OWNERSHIP_DENIED" });
    expect(() => manager.setPluginEnabled("node-1", "RpcServer", true)).toThrow(/observe-only/);

    expect(manager.pluginManager.installPlugin).not.toHaveBeenCalled();
    expect(manager.pluginManager.updatePluginConfig).not.toHaveBeenCalled();
    expect(manager.pluginManager.uninstallPlugin).not.toHaveBeenCalled();
    expect(manager.pluginManager.setPluginEnabled).not.toHaveBeenCalled();
  });

  it("resets node data by stopping running nodes and deleting each node", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getAllNodes: ReturnType<typeof vi.fn>;
      stopAllNodes: ReturnType<typeof vi.fn>;
      deleteNode: ReturnType<typeof vi.fn>;
    };

    manager.getAllNodes = vi.fn(() => [
      { id: "node-1", paths: { base: `${paths.nodes}/node-1` } },
      { id: "node-2", paths: { base: `${paths.nodes}/node-2` } },
    ]);
    manager.stopAllNodes = vi.fn().mockResolvedValue({
      stoppedCount: 1,
      alreadyStoppedCount: 1,
    });
    manager.deleteNode = vi.fn().mockResolvedValue(undefined);

    const result = await manager.resetAllNodeData();

    expect(manager.stopAllNodes).toHaveBeenCalledOnce();
    expect(manager.deleteNode).toHaveBeenCalledTimes(2);
    expect(result).toEqual({
      deletedNodeCount: 2,
      removedDirectoryCount: 2,
      stoppedCount: 1,
      alreadyStoppedCount: 1,
    });
  });

  it("does not abort reset-all or remove external files for running observe-only imported nodes", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      getAllNodes: ReturnType<typeof vi.fn>;
      stopNode: ReturnType<typeof vi.fn>;
      deleteNode: ReturnType<typeof vi.fn>;
    };

    manager.getAllNodes = vi.fn(() => [
      {
        id: "observed",
        paths: { base: "/opt/external-neo" },
        process: { status: "running" },
        settings: { import: { imported: true, ownershipMode: "observe-only", existingPath: "/opt/external-neo", importedAt: 1 } },
      },
    ]);
    manager.stopNode = vi
      .fn()
      .mockRejectedValue(Object.assign(new Error("denied"), { code: "NODE_OWNERSHIP_DENIED" }));
    manager.deleteNode = vi.fn().mockResolvedValue(undefined);

    const result = await manager.resetAllNodeData();

    expect(manager.stopNode).toHaveBeenCalledWith("observed");
    expect(manager.deleteNode).toHaveBeenCalledWith("observed");
    expect(rmSync).not.toHaveBeenCalled();
    expect(result).toEqual({
      deletedNodeCount: 1,
      removedDirectoryCount: 0,
      stoppedCount: 0,
      alreadyStoppedCount: 1,
    });
  });

  it("restores nodes from a snapshot and reinstalls exported plugins", async () => {
    const manager = Object.create(NodeManager.prototype) as NodeManager & {
      createNode: ReturnType<typeof vi.fn>;
      installPlugin: ReturnType<typeof vi.fn>;
      resetAllNodeData: ReturnType<typeof vi.fn>;
    };

    manager.createNode = vi
      .fn()
      .mockResolvedValueOnce({ id: "new-node-1", type: "neo-cli" })
      .mockResolvedValueOnce({ id: "new-node-2", type: "neo-go" });
    manager.installPlugin = vi.fn().mockResolvedValue(undefined);
    manager.resetAllNodeData = vi.fn().mockResolvedValue(undefined);

    const result = await manager.restoreConfiguration(
      {
        version: "2.0.0",
        nodes: [
          {
            name: "Node A",
            type: "neo-cli",
            network: "mainnet",
            syncMode: "full",
            version: "3.9.2",
            ports: { rpc: 10332, p2p: 10333 },
            settings: { debugMode: true },
            plugins: [{ id: "RpcServer", config: { Port: 10332 } }],
          },
          {
            name: "Node B",
            type: "neo-go",
            network: "testnet",
            syncMode: "light",
            version: "0.106.0",
            ports: { rpc: 20332, p2p: 20333 },
            settings: { relay: false },
            plugins: [],
          },
        ],
      },
      { replaceExisting: true },
    );

    expect(manager.resetAllNodeData).toHaveBeenCalledOnce();
    expect(manager.createNode).toHaveBeenCalledTimes(2);
    expect(manager.createNode).toHaveBeenNthCalledWith(1, {
      name: "Node A",
      type: "neo-cli",
      network: "mainnet",
      syncMode: "full",
      version: "3.9.2",
      customPorts: { rpc: 10332, p2p: 10333 },
      settings: { debugMode: true },
    });
    expect(manager.installPlugin).toHaveBeenCalledWith("new-node-1", "RpcServer", { Port: 10332 });
    expect(result).toEqual({
      restoredCount: 2,
      skippedCount: 0,
      failedCount: 0,
    });
  });
});
