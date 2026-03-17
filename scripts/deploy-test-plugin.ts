import { renderPluginConfig } from '../dashboard/src/services/plugins/PluginConfigRenderer';
import { buildRemotePluginSyncCommand } from '../dashboard/src/services/plugins/RemotePluginSync';
import { execSync } from 'child_process';
import * as fs from 'fs';

// 1. Simulate the DBFTPlugin Config from User Form
const mockPluginUserInput = {
  Network: 860833102,
  AutoStart: true,
  BlockTxNumber: 1024,
};

// 2. Render JSON using the same code the dashboard uses
const pluginJson = renderPluginConfig({
  pluginId: 'DBFTPlugin',
  endpointId: 42,
  secretRefs: ['DBFTPlugin_private_key'],
  secretPayloads: {
    'DBFTPlugin_private_key': 'L1Kmockkeythatisnotrealbutforvalidation'
  },
  runtimeImage: 'nginx:alpine', // Mock image that actually stays alive and can be pulled
  configData: mockPluginUserInput,
});

// 3. Create SSH Command
const sshCommand = buildRemotePluginSyncCommand({
  host: '91.99.197.255',
  user: 'root',
  identityPath: '/home/neo/.ssh/id_ed25519',
  remotePath: '/etc/neonexus/plugins/DBFTPlugin.json',
  renderedConfig: pluginJson,
});

console.log('Pushing Plugin Config via SSH...');
execSync(sshCommand, { stdio: 'inherit' });
console.log('Plugin deployed successfully!');
