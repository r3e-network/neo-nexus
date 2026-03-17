type RemoteSyncInput = {
  host: string;
  user: string;
  remotePath: string;
  renderedConfig: string;
  identityPath?: string;
};

export function buildRemotePluginSyncCommand(input: RemoteSyncInput): string {
  const encodedConfig = Buffer.from(input.renderedConfig, 'utf8').toString('base64');
  const remoteCommand = [
    `mkdir -p "$(dirname '${input.remotePath}')"`,
    `echo '${encodedConfig}' | base64 -d > '${input.remotePath}'`,
    'systemctl restart neonexus-plugin-sync',
  ].join(' && ');

  const identityFlag = input.identityPath ? ` -i ${input.identityPath}` : '';
  return `ssh${identityFlag} -o StrictHostKeyChecking=accept-new ${input.user}@${input.host} "${remoteCommand}"`;
}

export function buildRemotePluginRemovalCommand(input: Omit<RemoteSyncInput, 'renderedConfig'>): string {
  const remoteCommand = [
    `rm -f '${input.remotePath}'`,
    'systemctl restart neonexus-plugin-sync',
  ].join(' && ');

  const identityFlag = input.identityPath ? ` -i ${input.identityPath}` : '';
  return `ssh${identityFlag} -o StrictHostKeyChecking=accept-new ${input.user}@${input.host} "${remoteCommand}"`;
}
