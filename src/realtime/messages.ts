import type { LogEntry, NodeMetrics, NodeStatus, SystemMetrics, WebSocketMessage } from "../types";

export function buildSystemMessage(data: SystemMetrics): WebSocketMessage {
  return {
    type: "system",
    data,
    timestamp: Date.now(),
  };
}

export function buildNodeStatusMessage(
  nodeId: string,
  status: NodeStatus,
  previousStatus: NodeStatus,
): WebSocketMessage {
  return {
    type: "status",
    nodeId,
    data: {
      status,
      previousStatus,
    },
    timestamp: Date.now(),
  };
}

export function buildNodeLogMessage(nodeId: string, entry: LogEntry): WebSocketMessage {
  return {
    type: "log",
    nodeId,
    data: entry,
    timestamp: Date.now(),
  };
}

export function buildNodeMetricsMessage(nodeId: string, metrics: NodeMetrics): WebSocketMessage {
  return {
    type: "metrics",
    nodeId,
    data: metrics,
    timestamp: Date.now(),
  };
}
