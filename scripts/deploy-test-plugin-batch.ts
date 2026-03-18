import { renderPluginConfig } from '../dashboard/src/services/plugins/PluginConfigRenderer';
import { buildRemotePluginSyncCommand } from '../dashboard/src/services/plugins/RemotePluginSync';
import { execSync } from 'child_process';

const HOST = '91.99.197.255';
const SSH_KEY = '/home/neo/.ssh/id_ed25519';

const pluginsToTest = [
  { id: 'LevelDBStore', config: {} },
  { id: 'OracleService', config: { Network: 860833102, AutoStart: true }, secrets: { 'OracleService_private_key': 'mock_oracle_key' } },
  { id: 'RestServer', config: { Network: 860833102, BindAddress: '0.0.0.0', Port: 10334, KeepAliveTimeout: 60 } },
  { id: 'RocksDBStore', config: {} },
  { id: 'RpcServer', config: { DisabledMethods: ['getversion'], Network: 860833102, BindAddress: '0.0.0.0', Port: 10332, MaxConcurrentConnections: 40, KeepAliveTimeout: 60 } },
  { id: 'SignClient', config: {} },
  { id: 'SQLiteWallet', config: {} },
  { id: 'StateService', config: { Network: 860833102, AutoStart: true, FullState: false } },
  { id: 'StorageDumper', config: {} },
  { id: 'TokensTracker', config: { Network: 860833102 } },
];

for (const plugin of pluginsToTest) {
  console.log(`\n--- Simulating Deployment of ${plugin.id} ---`);
  
  const pluginJson = renderPluginConfig({
    pluginId: plugin.id,
    endpointId: 44, // mock ID
    secretRefs: plugin.secrets ? Object.keys(plugin.secrets) : [],
    secretPayloads: plugin.secrets || {},
    runtimeImage: 'nginx:alpine', // Using nginx so docker pull succeeds quickly
    configData: plugin.config,
  });

  const sshCommand = buildRemotePluginSyncCommand({
    host: HOST,
    user: 'root',
    identityPath: SSH_KEY,
    remotePath: `/etc/neonexus/plugins/${plugin.id}.json`,
    renderedConfig: pluginJson,
  });

  try {
    execSync(sshCommand, { stdio: 'inherit' });
    console.log(`✅ ${plugin.id} deployed successfully!`);
  } catch (error) {
    console.error(`❌ ${plugin.id} failed to deploy.`);
  }
}
