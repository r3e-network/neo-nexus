type RemoteSettingsInput = {
  host: string;
  user: string;
  identityPath?: string;
  settingsPath: string;
  renderedSettings: string;
  syncScriptPath: string;
  renderedSyncScript: string;
  neoGoConfigPath?: string;
  renderedNeoGoConfig?: string | null;
};

export function buildRemoteNodeSettingsSyncCommand(input: RemoteSettingsInput): string {
  const encodedSettings = Buffer.from(input.renderedSettings, 'utf8').toString('base64');
  const encodedSyncScript = Buffer.from(input.renderedSyncScript, 'utf8').toString('base64');
  const remoteCommand = [
    `mkdir -p "$(dirname '${input.settingsPath}')"`,
    `echo '${encodedSettings}' | base64 -d > '${input.settingsPath}'`,
    ...(input.neoGoConfigPath && input.renderedNeoGoConfig
      ? [
          `mkdir -p "$(dirname '${input.neoGoConfigPath}')"`,
          `echo '${Buffer.from(input.renderedNeoGoConfig, 'utf8').toString('base64')}' | base64 -d > '${input.neoGoConfigPath}'`,
        ]
      : []),
    `mkdir -p "$(dirname '${input.syncScriptPath}')"`,
    `echo '${encodedSyncScript}' | base64 -d > '${input.syncScriptPath}'`,
    `chmod +x '${input.syncScriptPath}'`,
    `${input.syncScriptPath}`,
  ].join(' && ');

  const identityFlag = input.identityPath ? ` -i ${input.identityPath}` : '';
  return `ssh${identityFlag} -o StrictHostKeyChecking=accept-new ${input.user}@${input.host} "${remoteCommand}"`;
}
