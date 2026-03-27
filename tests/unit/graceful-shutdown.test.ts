/**
 * Unit Tests: Graceful Shutdown & PID File Utilities
 *
 * Tests for src/utils/lifecycle.ts
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { join } from "node:path";
import { tmpdir } from "node:os";

// We do NOT mock node:fs here — we want real filesystem behaviour so
// writePidFile / removePidFile / readPidFile can actually be exercised.
// The global setup.ts mocks are scoped per-file via vi.mock hoisting but
// this file deliberately avoids those mocks for node:fs.

import {
  writePidFile,
  removePidFile,
  readPidFile,
  isProcessAlive,
} from "../../src/utils/lifecycle";

import * as fs from "node:fs";

// ------------------------------------------------------------------ helpers

function tmpPidPath(suffix = ""): string {
  return join(tmpdir(), `neonexus-test-pid-${Date.now()}${suffix}.pid`);
}

// ------------------------------------------------------------------ tests

describe("lifecycle: writePidFile", () => {
  it("writes the current process PID to the specified file", () => {
    const pidPath = tmpPidPath("write");
    try {
      writePidFile(pidPath);
      const contents = fs.readFileSync(pidPath, "utf-8").trim();
      expect(Number(contents)).toBe(process.pid);
    } finally {
      fs.rmSync(pidPath, { force: true });
    }
  });

  it("overwrites an existing PID file", () => {
    const pidPath = tmpPidPath("overwrite");
    try {
      fs.writeFileSync(pidPath, "99999", "utf-8");
      writePidFile(pidPath);
      const contents = fs.readFileSync(pidPath, "utf-8").trim();
      expect(Number(contents)).toBe(process.pid);
    } finally {
      fs.rmSync(pidPath, { force: true });
    }
  });
});

describe("lifecycle: removePidFile", () => {
  it("deletes an existing PID file", () => {
    const pidPath = tmpPidPath("remove");
    fs.writeFileSync(pidPath, "12345", "utf-8");
    expect(fs.existsSync(pidPath)).toBe(true);

    removePidFile(pidPath);
    expect(fs.existsSync(pidPath)).toBe(false);
  });

  it("does not throw when the file does not exist", () => {
    const pidPath = tmpPidPath("missing");
    expect(() => removePidFile(pidPath)).not.toThrow();
  });
});

describe("lifecycle: readPidFile", () => {
  it("returns the PID number from an existing file", () => {
    const pidPath = tmpPidPath("read");
    try {
      fs.writeFileSync(pidPath, "42000\n", "utf-8");
      const pid = readPidFile(pidPath);
      expect(pid).toBe(42000);
    } finally {
      fs.rmSync(pidPath, { force: true });
    }
  });

  it("returns null when the file does not exist", () => {
    const pidPath = tmpPidPath("readmissing");
    const pid = readPidFile(pidPath);
    expect(pid).toBeNull();
  });

  it("returns null when the file contains non-numeric content", () => {
    const pidPath = tmpPidPath("readbad");
    try {
      fs.writeFileSync(pidPath, "not-a-number", "utf-8");
      const pid = readPidFile(pidPath);
      expect(pid).toBeNull();
    } finally {
      fs.rmSync(pidPath, { force: true });
    }
  });
});

describe("lifecycle: isProcessAlive", () => {
  it("returns true for the current process PID", () => {
    expect(isProcessAlive(process.pid)).toBe(true);
  });

  it("returns false for a PID that is not alive", () => {
    // PID 0 is not a real user process; sending signal 0 to it throws EPERM
    // or succeeds depending on OS, so we use a clearly dead PID instead.
    // We look for a PID that is definitely not running by trying very large
    // numbers unlikely to be in use.
    //
    // More deterministically: process.kill throws ESRCH when the process
    // does not exist — isProcessAlive should catch that and return false.
    // We spy on process.kill to simulate that.
    const originalKill = process.kill.bind(process);
    const killSpy = vi.spyOn(process, "kill").mockImplementationOnce((pid, signal) => {
      if (pid === 999999 && signal === 0) {
        const err = Object.assign(new Error("No such process"), { code: "ESRCH" });
        throw err;
      }
      return originalKill(pid as number, signal as NodeJS.Signals);
    });

    try {
      expect(isProcessAlive(999999)).toBe(false);
    } finally {
      killSpy.mockRestore();
    }
  });

  it("returns false when ESRCH error is thrown", () => {
    const killSpy = vi.spyOn(process, "kill").mockImplementationOnce(() => {
      throw Object.assign(new Error("ESRCH"), { code: "ESRCH" });
    });
    try {
      expect(isProcessAlive(1)).toBe(false);
    } finally {
      killSpy.mockRestore();
    }
  });
});
