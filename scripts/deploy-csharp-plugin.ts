import { renderPluginConfig } from '../dashboard/src/services/plugins/PluginConfigRenderer';
import { buildRemotePluginSyncCommand } from '../dashboard/src/services/plugins/RemotePluginSync';
import { execSync } from 'child_process';

const pluginJson = renderPluginConfig({
  pluginId: 'ApplicationLogs',
  endpointId: 43,
  secretRefs: [],
  secretPayloads: {},
  runtimeImage: 'alpine', // Mocking since actual plugin images require specific packaging
  configData: {
    Network: 860833102,
    MaxLogSize: 500000,
  },
});

const sshCommand = buildRemotePluginSyncCommand({
  host: '91.99.197.255',
  user: 'root',
  identityPath: '/home/neo/.ssh/id_ed25519',
  remotePath: '/etc/neonexus/plugins/ApplicationLogs.json',
  renderedConfig: pluginJson,
});

console.log('Pushing C# Plugin Config (ApplicationLogs) via SSH...');
execSync(sshCommand, { stdio: 'inherit' });
console.log('Plugin deployed successfully!');
