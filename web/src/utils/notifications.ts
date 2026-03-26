import type { WebSocketMessage } from "../../../src/types";

export interface AppNotification {
  id: string;
  nodeId?: string;
  title: string;
  message: string;
  level: "info" | "success" | "warning" | "error";
  createdAt: number;
  read?: boolean;
}

export function notificationFromRealtimeMessage(message: WebSocketMessage): AppNotification | null {
  if (message.type === "log") {
    const data = message.data as {
      timestamp: number;
      level: "debug" | "info" | "warn" | "error";
      message: string;
    };

    if (data.level !== "error" && data.level !== "warn") {
      return null;
    }

    return {
      id: `log:${message.nodeId}:${data.timestamp}:${data.level}:${data.message}`,
      nodeId: message.nodeId,
      title: data.level === "error" ? "Node Error" : "Node Warning",
      message: data.message,
      level: data.level === "error" ? "error" : "warning",
      createdAt: message.timestamp,
    };
  }

  if (message.type === "status") {
    const data = message.data as {
      status: "stopped" | "starting" | "running" | "stopping" | "error" | "syncing";
      previousStatus: "stopped" | "starting" | "running" | "stopping" | "error" | "syncing";
    };

    if (!["running", "stopped", "error"].includes(data.status)) {
      return null;
    }

    const title =
      data.status === "error" ? "Node Error" : data.status === "running" ? "Node Running" : "Node Stopped";
    const level = data.status === "error" ? "error" : data.status === "running" ? "success" : "warning";

    return {
      id: `status:${message.nodeId}:${data.status}:${message.timestamp}`,
      nodeId: message.nodeId,
      title,
      message: `Node ${message.nodeId} changed from ${data.previousStatus} to ${data.status}.`,
      level,
      createdAt: message.timestamp,
    };
  }

  return null;
}

export function dedupeNotifications(notifications: AppNotification[]): AppNotification[] {
  const deduped = new Map<string, AppNotification>();

  for (const notification of notifications.sort((left, right) => right.createdAt - left.createdAt)) {
    if (!deduped.has(notification.id)) {
      deduped.set(notification.id, notification);
    }
  }

  return [...deduped.values()].sort((left, right) => right.createdAt - left.createdAt);
}
