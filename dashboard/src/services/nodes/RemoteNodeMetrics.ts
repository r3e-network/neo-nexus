import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);

type SshInput = {
  host: string;
  user: string;
  identityPath?: string;
};

type MetricsDependencies = {
  execFileImpl?: typeof execFileAsync;
};

export type NodeMetricsSnapshot = {
  cpuPercent: number | null;
  memoryUsedBytes: number | null;
  memoryLimitBytes: number | null;
  networkRxBytes: number | null;
  networkTxBytes: number | null;
  blockReadBytes: number | null;
  blockWriteBytes: number | null;
  diskUsedBytes: number | null;
  diskTotalBytes: number | null;
};

function buildSshCommand(input: SshInput, remoteCommand: string) {
  const identityFlag = input.identityPath ? ` -i ${input.identityPath}` : '';
  return `ssh${identityFlag} -o StrictHostKeyChecking=accept-new ${input.user}@${input.host} "${remoteCommand}"`;
}

export function buildRemoteNodeMetricsCommand(input: SshInput): string {
  const remoteCommand = [
    `docker stats neonexus-node --no-stream --format '{{json .}}' 2>/dev/null || echo '{}'`,
    `df -B1 /var/lib/neonexus | tail -1 | awk '{print \\$3","\\$2}'`,
  ].join('; ');

  return buildSshCommand(input, remoteCommand);
}

function parseSizeToBytes(value: string): number | null {
  const normalized = value.trim().replace(/iB$/, 'iB').replace(/B$/, 'B');
  const match = normalized.match(/^([\d.]+)\s*([KMGTP]?i?B)$/i);
  if (!match) {
    return null;
  }

  const amount = Number.parseFloat(match[1]);
  const unit = match[2].toUpperCase();
  const multipliers: Record<string, number> = {
    B: 1,
    KB: 1_000,
    MB: 1_000_000,
    GB: 1_000_000_000,
    TB: 1_000_000_000_000,
    KIB: 1024,
    MIB: 1024 ** 2,
    GIB: 1024 ** 3,
    TIB: 1024 ** 4,
  };

  return Math.round(amount * (multipliers[unit] ?? 1));
}

function parsePair(value: string): [number | null, number | null] {
  const [left, right] = value.split('/').map((part) => part.trim());
  return [parseSizeToBytes(left), parseSizeToBytes(right)];
}

export function parseNodeMetricsSnapshot(rawOutput: string): NodeMetricsSnapshot {
  const [dockerLine = '{}', diskLine = ''] = rawOutput.trim().split('\n');
  const diskParts = diskLine.split(',');

  const dockerStats = JSON.parse(dockerLine) as {
    CPUPerc?: string;
    MemUsage?: string;
    NetIO?: string;
    BlockIO?: string;
  };

  const [memoryUsedBytes, memoryLimitBytes] = dockerStats.MemUsage ? parsePair(dockerStats.MemUsage) : [null, null];
  const [networkRxBytes, networkTxBytes] = dockerStats.NetIO ? parsePair(dockerStats.NetIO) : [null, null];
  const [blockReadBytes, blockWriteBytes] = dockerStats.BlockIO ? parsePair(dockerStats.BlockIO) : [null, null];

  return {
    cpuPercent: dockerStats.CPUPerc ? Number.parseFloat(dockerStats.CPUPerc.replace('%', '')) : null,
    memoryUsedBytes,
    memoryLimitBytes,
    networkRxBytes,
    networkTxBytes,
    blockReadBytes,
    blockWriteBytes,
    diskUsedBytes: diskParts[0] ? Number.parseInt(diskParts[0], 10) : null,
    diskTotalBytes: diskParts[1] ? Number.parseInt(diskParts[1], 10) : null,
  };
}

async function runSshCommand(command: string, dependencies: MetricsDependencies = {}) {
  const runner = dependencies.execFileImpl ?? execFileAsync;
  const result = await runner('/bin/bash', ['-lc', command]);
  return result.stdout.trim();
}

export async function fetchRemoteNodeMetrics(
  input: SshInput,
  dependencies: MetricsDependencies = {},
) {
  const stdout = await runSshCommand(buildRemoteNodeMetricsCommand(input), dependencies);
  return parseNodeMetricsSnapshot(stdout);
}
