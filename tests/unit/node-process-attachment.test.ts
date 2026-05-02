import { beforeEach, describe, expect, it, vi } from "vitest";
import type { NodeInstance } from "../../src/types";

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

import {
  getAttachedProcessState,
  isManagedNodeDirectory,
  isPathWithinOrEqual,
  isValidProcessId,
  parseProcessIds,
  scoreAttachCandidate,
} from "../../src/core/nodeProcessAttachment";
import * as lifecycle from "../../src/utils/lifecycle";

const node = {
  id: "node-test",
  type: "neo-cli",
  ports: { rpc: 20332, p2p: 20333, websocket: 20334, metrics: 20335 },
  paths: {
    base: "/opt/neo",
    config: "/opt/neo/config.testnet.json",
    data: "/opt/neo/Chain",
    logs: "/opt/neo/Logs",
  },
  process: { status: "running", pid: 222 },
  settings: { import: { imported: true, ownershipMode: "managed-process" } },
} as NodeInstance;

describe("node process attachment helpers", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.mocked(lifecycle.isProcessAlive).mockReset();
    vi.mocked(lifecycle.getProcessCommand).mockReset();
    vi.mocked(lifecycle.getProcessCwd).mockReset();
    vi.mocked(lifecycle.getProcessArgv).mockReset();
  });

  it("parses unique safe process IDs from pgrep output", () => {
    expect(parseProcessIds("111\n222\n111\n0\nbad\n9007199254740992\n")).toEqual([111, 222]);
  });

  it("rejects invalid process IDs", () => {
    expect(isValidProcessId(1)).toBe(true);
    expect(isValidProcessId(0)).toBe(false);
    expect(isValidProcessId(-1)).toBe(false);
    expect(isValidProcessId(1.5)).toBe(false);
    expect(isValidProcessId("123")).toBe(false);
  });

  it("checks whether a path is inside or equal to an allowed root", () => {
    expect(isPathWithinOrEqual("/opt/neo", "/opt/neo")).toBe(true);
    expect(isPathWithinOrEqual("/opt/neo/Chain", "/opt/neo")).toBe(true);
    expect(isPathWithinOrEqual("/opt/neo-other", "/opt/neo")).toBe(false);
  });

  it("recognizes generated NeoNexus managed node directories only under the managed root", () => {
    expect(isManagedNodeDirectory({
      id: "node-abc-123",
      paths: { base: "/data/neonexus/nodes/node-abc-123" },
      settings: {},
    }, "/data/neonexus/nodes")).toBe(true);

    expect(isManagedNodeDirectory({
      id: "node-abc-123",
      paths: { base: "/opt/external/node-abc-123" },
      settings: {},
    }, "/data/neonexus/nodes")).toBe(false);

    expect(isManagedNodeDirectory({
      id: "node-abc-123",
      paths: { base: "/data/neonexus/nodes/node-abc-123" },
      settings: { import: { imported: true, ownershipMode: "observe-only" } },
    }, "/data/neonexus/nodes")).toBe(false);
  });

  it("scores a candidate by command, cwd, config path, node path, and ports", () => {
    vi.mocked(lifecycle.isProcessAlive).mockReturnValue(true);
    vi.mocked(lifecycle.getProcessCommand).mockReturnValue("dotnet /opt/neo/neo-cli.dll --config-path /opt/neo/config.testnet.json");
    vi.mocked(lifecycle.getProcessCwd).mockReturnValue("/opt/neo");
    vi.mocked(lifecycle.getProcessArgv).mockReturnValue([
      "dotnet",
      "/opt/neo/neo-cli.dll",
      "--config-path=/opt/neo/config.testnet.json",
      "--rpc-port",
      "20332",
    ]);

    expect(scoreAttachCandidate(node, 222)).toBe(18);
  });

  it("marks an attached process active only when it is alive and belongs to the node", () => {
    vi.mocked(lifecycle.isProcessAlive).mockReturnValue(true);
    vi.mocked(lifecycle.getProcessCommand).mockReturnValue("dotnet /opt/neo/neo-cli.dll");
    vi.mocked(lifecycle.getProcessCwd).mockReturnValue("/opt/neo");
    vi.mocked(lifecycle.getProcessArgv).mockReturnValue(["dotnet", "/opt/neo/neo-cli.dll"]);

    expect(getAttachedProcessState(node, 222)).toBe("active");

    vi.mocked(lifecycle.getProcessCwd).mockReturnValue("/tmp/unrelated");
    vi.mocked(lifecycle.getProcessArgv).mockReturnValue(["python", "/tmp/unrelated.py"]);

    expect(getAttachedProcessState(node, 222)).toBe("stale");
  });
});
