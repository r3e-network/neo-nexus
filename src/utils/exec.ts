import { spawn, type ChildProcess } from 'node:child_process';
import { promisify } from 'node:util';
import { exec as execCallback } from 'node:child_process';

export const execAsync = promisify(execCallback);

export interface ProcessResult {
  stdout: string;
  stderr: string;
  exitCode: number;
}

export async function exec(command: string, args: string[] = [], options?: { cwd?: string; env?: NodeJS.ProcessEnv }): Promise<ProcessResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: options?.cwd,
      env: { ...process.env, ...options?.env },
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';

    child.stdout?.on('data', (data) => {
      stdout += data.toString();
    });

    child.stderr?.on('data', (data) => {
      stderr += data.toString();
    });

    child.on('close', (code) => {
      resolve({
        stdout: stdout.trim(),
        stderr: stderr.trim(),
        exitCode: code ?? 0,
      });
    });

    child.on('error', (error) => {
      reject(error);
    });
  });
}

export async function execShell(command: string): Promise<ProcessResult> {
  const { stdout, stderr } = await execAsync(command);
  return {
    stdout: stdout.trim(),
    stderr: stderr.trim(),
    exitCode: 0,
  };
}

export function spawnProcess(
  command: string,
  args: string[],
  options?: { cwd?: string; env?: NodeJS.ProcessEnv; logCallback?: (line: string) => void }
): ChildProcess {
  const child = spawn(command, args, {
    cwd: options?.cwd,
    env: { ...process.env, ...options?.env },
    stdio: ['pipe', 'pipe', 'pipe'],
    detached: false,
  });

  if (options?.logCallback) {
    child.stdout?.on('data', (data) => {
      const lines = data.toString().split('\n');
      for (const line of lines) {
        if (line.trim()) {
          options.logCallback?.(line.trim());
        }
      }
    });

    child.stderr?.on('data', (data) => {
      const lines = data.toString().split('\n');
      for (const line of lines) {
        if (line.trim()) {
          options.logCallback?.(`[stderr] ${line.trim()}`);
        }
      }
    });
  }

  return child;
}

export async function checkCommandExists(command: string): Promise<boolean> {
  try {
    await execAsync(`which ${command}`);
    return true;
  } catch {
    return false;
  }
}

export async function getProcessInfo(pid: number): Promise<{ pid: number; name: string; cpu: number; memory: number } | null> {
  try {
    const { stdout } = await execAsync(`ps -p ${pid} -o pid,comm,pcpu,pmem --no-headers`);
    const parts = stdout.trim().split(/\s+/);
    if (parts.length >= 4) {
      return {
        pid: parseInt(parts[0], 10),
        name: parts[1],
        cpu: parseFloat(parts[2]),
        memory: parseFloat(parts[3]),
      };
    }
    return null;
  } catch {
    return null;
  }
}

export async function killProcess(pid: number, force = false): Promise<boolean> {
  try {
    const signal = force ? 'SIGKILL' : 'SIGTERM';
    process.kill(pid, signal);
    return true;
  } catch {
    return false;
  }
}
