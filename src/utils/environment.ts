import { existsSync, readFileSync } from "node:fs";

export interface RuntimeEnvironment {
  isDocker: boolean;
  isSystemd: boolean;
}

export function detectEnvironment(): RuntimeEnvironment {
  return {
    isDocker: existsSync("/.dockerenv") || checkCgroup(),
    isSystemd: !!process.env.INVOCATION_ID,
  };
}

function checkCgroup(): boolean {
  try {
    return readFileSync("/proc/1/cgroup", "utf8").includes("docker");
  } catch {
    return false;
  }
}
