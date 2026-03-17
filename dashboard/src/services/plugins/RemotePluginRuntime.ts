import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);

type SshInput = {
  host: string;
  user: string;
  identityPath?: string;
  pluginId: string;
};

type RuntimeDependencies = {
  execFileImpl?: typeof execFileAsync;
};

type RuntimeStatus = {
  status: 'Active' | 'Pending Apply' | 'Error';
  errorMessage: string | null;
};

export function getPluginContainerName(pluginId: string): string {
  return `neonexus-plugin-${pluginId}`;
}

function buildSshCommand(input: SshInput, remoteCommand: string) {
  const identityFlag = input.identityPath ? ` -i ${input.identityPath}` : '';
  return `ssh${identityFlag} -o StrictHostKeyChecking=accept-new ${input.user}@${input.host} "${remoteCommand}"`;
}

export function buildRemotePluginStatusCommand(input: SshInput): string {
  return buildSshCommand(
    input,
    `docker inspect ${getPluginContainerName(input.pluginId)} --format '{{json .State}}' 2>/dev/null || echo '{}'`,
  );
}

export function buildRemotePluginLogsCommand(input: SshInput): string {
  return buildSshCommand(
    input,
    `docker logs --tail 200 ${getPluginContainerName(input.pluginId)} 2>&1 || echo 'Plugin container not found'`,
  );
}

export function parsePluginRuntimeState(rawState: string): RuntimeStatus {
  const trimmed = rawState.trim();
  if (!trimmed || trimmed === '{}') {
    return {
      status: 'Pending Apply',
      errorMessage: 'Plugin container is not present on the node yet.',
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
        errorMessage: 'Plugin container is still starting.',
      };
    }

    const suffix = state.Error ? ` ${state.Error}` : '';
    return {
      status: 'Error',
      errorMessage: `Plugin container exited with code ${state.ExitCode ?? 'unknown'}.${suffix}`.trim(),
    };
  } catch {
    return {
      status: 'Error',
      errorMessage: 'Unable to parse plugin runtime state from the managed node.',
    };
  }
}

async function runSshCommand(command: string, dependencies: RuntimeDependencies = {}) {
  const runner = dependencies.execFileImpl ?? execFileAsync;
  const result = await runner('/bin/bash', ['-lc', command]);
  return result.stdout.trim();
}

export async function fetchRemotePluginRuntimeStatus(
  input: SshInput,
  dependencies: RuntimeDependencies = {},
) {
  const stdout = await runSshCommand(buildRemotePluginStatusCommand(input), dependencies);
  return parsePluginRuntimeState(stdout);
}

export async function fetchRemotePluginLogs(
  input: SshInput,
  dependencies: RuntimeDependencies = {},
) {
  return runSshCommand(buildRemotePluginLogsCommand(input), dependencies);
}
