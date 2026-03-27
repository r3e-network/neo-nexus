/**
 * Unit Tests: Zombie Process Detection
 *
 * Tests for getProcessCommand in src/utils/lifecycle.ts
 *
 * We deliberately unmock node:child_process so execSync runs for real,
 * matching the pattern used in graceful-shutdown.test.ts for node:fs.
 */

import { describe, it, expect, vi } from "vitest";

// Restore the real execSync before the module under test is loaded
vi.unmock("node:child_process");
vi.unmock("child_process");

import { getProcessCommand } from "../../src/utils/lifecycle";

describe("lifecycle: getProcessCommand", () => {
  it("returns a non-empty string for the current process PID", () => {
    const cmd = getProcessCommand(process.pid);
    expect(cmd).not.toBeNull();
    expect(typeof cmd).toBe("string");
    expect((cmd as string).length).toBeGreaterThan(0);
  });

  it("returns null for a dead PID (9999999)", () => {
    const cmd = getProcessCommand(9999999);
    expect(cmd).toBeNull();
  });
});
