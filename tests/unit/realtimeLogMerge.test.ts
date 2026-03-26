import { describe, expect, it } from "vitest";
import { mergeNodeLogs } from "../../web/src/utils/realtime";

describe("mergeNodeLogs", () => {
  it("appends new realtime log entries without duplicating polled entries", () => {
    const merged = mergeNodeLogs(
      [
        { timestamp: 200, level: "info", message: "newest" },
        { timestamp: 100, level: "warn", message: "older" },
      ],
      [
        { timestamp: 200, level: "info", message: "newest" },
        { timestamp: 300, level: "error", message: "live" },
      ],
    );

    expect(merged).toEqual([
      { timestamp: 100, level: "warn", message: "older" },
      { timestamp: 200, level: "info", message: "newest" },
      { timestamp: 300, level: "error", message: "live" },
    ]);
  });
});
