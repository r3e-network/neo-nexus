type SshInput = {
  host: string;
  user: string;
  identityPath?: string;
};

function buildSshCommand(input: SshInput, remoteCommand: string) {
  const identityFlag = input.identityPath ? ` -i ${input.identityPath}` : '';
  return `ssh${identityFlag} -o StrictHostKeyChecking=accept-new ${input.user}@${input.host} "${remoteCommand}"`;
}

export function buildRemoteResyncCommand(input: SshInput): string {
  const remoteCommand = [
    'docker stop neonexus-node || true',
    'rm -rf /var/lib/neonexus/*',
    'docker start neonexus-node',
  ].join(' && ');

  return buildSshCommand(input, remoteCommand);
}

export function buildRemoteSnapshotCommand(
  input: SshInput & { snapshotPath: string },
) {
  const remoteCommand = [
    `mkdir -p "$(dirname '${input.snapshotPath}')"`,
    `tar -czf '${input.snapshotPath}' -C /var/lib neonexus`,
    `echo '${input.snapshotPath}'`,
  ].join(' && ');

  return buildSshCommand(input, remoteCommand);
}
