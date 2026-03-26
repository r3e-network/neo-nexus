import { describe, expect, it } from "vitest";
import type { WebSocketMessage } from "../../src/types";
import { dedupeNotifications, notificationFromRealtimeMessage } from "../../web/src/utils/notifications";

describe("notificationFromRealtimeMessage", () => {
  it("creates a high-severity notification for error log events", () => {
    const message: WebSocketMessage = {
      type: "log",
      nodeId: "node-1",
      timestamp: 123,
      data: {
        timestamp: 123,
        level: "error",
        source: "neo-cli",
        message: "Consensus fault",
      },
    };

    expect(notificationFromRealtimeMessage(message)).toEqual({
      id: "log:node-1:123:error:Consensus fault",
      nodeId: "node-1",
      title: "Node Error",
      message: "Consensus fault",
      level: "error",
      createdAt: 123,
    });
  });

  it("creates a notification for node status transitions", () => {
    const message: WebSocketMessage = {
      type: "status",
      nodeId: "node-2",
      timestamp: 456,
      data: {
        status: "running",
        previousStatus: "starting",
      },
    };

    expect(notificationFromRealtimeMessage(message)).toEqual({
      id: "status:node-2:running:456",
      nodeId: "node-2",
      title: "Node Running",
      message: "Node node-2 changed from starting to running.",
      level: "success",
      createdAt: 456,
    });
  });

  it("ignores non-alert realtime messages", () => {
    const message: WebSocketMessage = {
      type: "metrics",
      nodeId: "node-1",
      timestamp: 789,
      data: { blockHeight: 100 },
    };

    expect(notificationFromRealtimeMessage(message)).toBeNull();
  });
});

describe("dedupeNotifications", () => {
  it("keeps the newest notification first and removes duplicates by id", () => {
    expect(
      dedupeNotifications([
        {
          id: "same",
          title: "New",
          message: "latest",
          level: "warning",
          createdAt: 200,
        },
        {
          id: "same",
          title: "Old",
          message: "stale",
          level: "warning",
          createdAt: 100,
        },
        {
          id: "other",
          title: "Other",
          message: "keep",
          level: "info",
          createdAt: 150,
        },
      ]),
    ).toEqual([
      {
        id: "same",
        title: "New",
        message: "latest",
        level: "warning",
        createdAt: 200,
      },
      {
        id: "other",
        title: "Other",
        message: "keep",
        level: "info",
        createdAt: 150,
      },
    ]);
  });
});
