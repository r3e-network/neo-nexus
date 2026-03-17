import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);

type SshInput = {
  host: string;
  user: string;
  identityPath?: string;
};

type RuntimeDependencies = {
  execFileImpl?: typeof execFileAsync;
};

type RuntimeStatus = {
  status: 'Active' | 'Pending Apply' | 'Error';
  errorMessage: string | null;
};

function buildSshCommand(input: SshInput, remoteCommand: string) {
  const identityFlag = input.identityPath ? ` -i ${input.identityPath}` : '';
  return `ssh${identityFlag} -o StrictHostKeyChecking=accept-new ${input.user}@${input.host} "${remoteCommand}"`;
}

export function buildRemoteNodeStatusCommand(input: SshInput): string {
  return buildSshCommand(
    input,
    `docker inspect neonexus-node --format '{{json .State}}' 2>/dev/null || echo '{}'`,
  );
}

export function buildRemoteNodeLogsCommand(input: SshInput): string {
  return buildSshCommand(
    input,
    `docker logs --tail 400 neonexus-node 2>&1 || echo 'Node container not found'`,
  );
}

export function parseNodeRuntimeState(rawState: string): RuntimeStatus {
  const trimmed = rawState.trim();
  if (!trimmed || trimmed === '{}') {
    return {
      status: 'Pending Apply',
      errorMessage: 'Node container is not present on the VM yet.',
    };
  }

  try {
    const state = JSON.parse(trimmed) as {
      Running?: boolean;
      Status?: string;
      ExitCode?: number;
      Error?: string;
    };

    if (state.Running || state.Status === 'running') {
      return {
        status: 'Active',
        errorMessage: null,
      };
    }

    if (state.Status === 'created' || state.Status === 'restarting') {
      return {
        status: 'Pending Apply',
        errorMessage: 'Node container is still starting.',
      };
    }

    const suffix = state.Error ? ` ${state.Error}` : '';
    return {
      status: 'Error',
      errorMessage: `Node container exited with code ${state.ExitCode ?? 'unknown'}.${suffix}`.trim(),
    };
  } catch {
    return {
      status: 'Error',
      errorMessage: 'Unable to parse node runtime state from the managed VM.',
    };
  }
}

async function runSshCommand(command: string, dependencies: RuntimeDependencies = {}) {
  const runner = dependencies.execFileImpl ?? execFileAsync;
  const result = await runner('/bin/bash', ['-lc', command]);
  return result.stdout.trim();
}

export async function fetchRemoteNodeRuntimeStatus(
  input: SshInput,
  dependencies: RuntimeDependencies = {},
) {
  const stdout = await runSshCommand(buildRemoteNodeStatusCommand(input), dependencies);
  return parseNodeRuntimeState(stdout);
}

export async function fetchRemoteNodeLogs(
  input: SshInput,
  dependencies: RuntimeDependencies = {},
) {
  return runSshCommand(buildRemoteNodeLogsCommand(input), dependencies);
}
