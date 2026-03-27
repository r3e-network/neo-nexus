/**
 * Process Lifecycle Utilities
 *
 * Helpers for PID file management and liveness checks used by the graceful
 * shutdown / signal-handling machinery in server.ts.
 */

import { writeFileSync, readFileSync, unlinkSync, existsSync } from "node:fs";
import { execSync } from "node:child_process";

/**
 * Write the current process PID to `path` (creates or overwrites the file).
 */
export function writePidFile(path: string): void {
  writeFileSync(path, String(process.pid), "utf-8");
}

/**
 * Delete the PID file at `path`.  Errors (e.g. file already gone) are
 * silently ignored so this is always safe to call during shutdown.
 */
export function removePidFile(path: string): void {
  try {
    unlinkSync(path);
  } catch {
    // Ignore — file may already be gone or never existed
  }
}

/**
 * Read a PID from a file.
 *
 * @returns The PID as a number, or `null` if the file does not exist or
 *          its contents cannot be parsed as a valid integer.
 */
export function readPidFile(path: string): number | null {
  if (!existsSync(path)) {
    return null;
  }
  try {
    const raw = readFileSync(path, "utf-8").trim();
    const pid = parseInt(raw, 10);
    if (isNaN(pid)) {
      return null;
    }
    return pid;
  } catch {
    return null;
  }
}

/**
 * Return the command string for a running process, or null if the process
 * does not exist or the command cannot be determined.
 */
export function getProcessCommand(pid: number): string | null {
  try {
    return execSync(`ps -p ${pid} -o args= 2>/dev/null`, { encoding: 'utf-8' }).trim() || null;
  } catch { return null; }
}

/**
 * Check whether a process with the given PID is currently alive by sending
 * signal 0 (a no-op that still validates the PID).
 *
 * @returns `true` if the process exists, `false` otherwise.
 */
export function isProcessAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch (err) {
    // ESRCH → no such process; EPERM → process exists but we lack permission
    if ((err as NodeJS.ErrnoException).code === "EPERM") {
      return true;
    }
    return false;
  }
}
