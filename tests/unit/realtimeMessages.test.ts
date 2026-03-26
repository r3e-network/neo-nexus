import { describe, expect, it } from "vitest";
import { buildNodeLogMessage, buildNodeMetricsMessage, buildNodeStatusMessage, buildSystemMessage } from "../../src/realtime/messages";

describe("realtime message builders", () => {
  it("builds a node status websocket message", () => {
    const message = buildNodeStatusMessage("node-1", "running", "starting");

    expect(message.type).toBe("status");
    expect(message.nodeId).toBe("node-1");
    expect(message.data).toEqual({
      status: "running",
      previousStatus: "starting",
    });
  });

  it("builds a node log websocket message", () => {
    const message = buildNodeLogMessage("node-1", {
      timestamp: 123,
      level: "info",
      source: "neo-cli",
      message: "Block persisted",
    });

    expect(message.type).toBe("log");
    expect(message.nodeId).toBe("node-1");
    expect(message.data).toEqual({
      timestamp: 123,
      level: "info",
      source: "neo-cli",
      message: "Block persisted",
    });
  });

  it("builds a metrics websocket message", () => {
    const message = buildNodeMetricsMessage("node-1", {
      blockHeight: 100,
      headerHeight: 100,
      connectedPeers: 8,
      unconnectedPeers: 0,
      syncProgress: 0,
      memoryUsage: 1,
      cpuUsage: 2,
      lastUpdate: 3,
    });

    expect(message.type).toBe("metrics");
    expect(message.nodeId).toBe("node-1");
    expect((message.data as any).blockHeight).toBe(100);
  });

  it("builds a system websocket message", () => {
    const message = buildSystemMessage({ message: "Connected" });

    expect(message.type).toBe("system");
    expect(message.data).toEqual({ message: "Connected" });
  });
});
