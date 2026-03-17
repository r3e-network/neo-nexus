import { describe, expect, it } from 'vitest';
import { getTemplateById } from '../services/provisioning/NodeTemplates';
import { buildNodeBootstrapScript } from '../services/provisioning/NodeBootstrap';
import { buildDefaultNodeSettings, mergeNodeSettings } from '../services/settings/NodeSettings';
import { renderNodeRuntimeArtifacts } from '../services/settings/RenderedNodeRuntime';
import { getPluginDefinition } from '../services/plugins/PluginCatalog';
import { renderPluginConfig } from '../services/plugins/PluginConfigRenderer';
import { buildRemotePluginSyncCommand } from '../services/plugins/RemotePluginSync';

describe('End-to-End Platform Dataflow Simulation', () => {
  it('successfully simulates the lifecycle of a dedicated Consensus Node', () => {
    // 1. User selects a Template in the UI
    const template = getTemplateById('consensus');
    expect(template).toBeDefined();
    expect(template?.recommendedEngine).toBe('neo-cli');
    expect(template?.defaultPlugins).toContain('DBFTPlugin');

    // 2. The Provisioning Service prepares the Cloud-Init script
    const bootstrapScript = buildNodeBootstrapScript({
      endpointName: 'Production-Consensus-1',
      protocol: 'neo-n3',
      clientEngine: 'neo-cli',
      network: 'mainnet',
      syncMode: 'full',
      operatorPublicKey: 'ssh-ed25519 AAAAC3... mock',
    });

    expect(bootstrapScript).toContain('#cloud-config');
    expect(bootstrapScript).toContain('neo-cli');
    expect(bootstrapScript).toContain('ssh-ed25519 AAAAC3... mock'); // VM operator key injected

    // 3. The Node boots and the user configures their NodeSettings in the UI
    const baseSettings = buildDefaultNodeSettings('neo-cli');
    const userSettings = mergeNodeSettings('neo-cli', {
      ...baseSettings,
      rpcEnabled: true,
      envVars: { 'NEO_DEBUG': '1' },
      customDockerFlags: '--memory=16g'
    });

    // 4. The RenderedNodeRuntime converts this to the SSH payload
    const runtime = renderNodeRuntimeArtifacts({
      clientEngine: 'neo-cli',
      network: 'mainnet',
      settings: userSettings,
    });

    expect(runtime.runCommand).toContain('--rpc'); // RPC was enabled
    expect(runtime.runCommand).toContain('-e NEO_DEBUG="1"'); // Env var injected
    expect(runtime.runCommand).toContain('--memory=16g'); // Custom flag injected
    expect(runtime.runCommand).toContain('docker run -d');

    // 5. The User configures the required DBFT Plugin
    const dbftDef = getPluginDefinition('DBFTPlugin');
    expect(dbftDef?.requiresPrivateKey).toBe(true);
    
    // Simulate what the UI sends to the server action
    const mockPluginUserInput = {
      Network: 860833102,
      AutoStart: true,
      BlockTxNumber: 1024, // User bumped it from default 512
    };

    // 6. Plugin Config Renderer builds the exact JSON file for the plugin
    const pluginJson = renderPluginConfig({
      pluginId: 'DBFTPlugin',
      endpointId: 42,
      secretRefs: ['DBFTPlugin_private_key'],
      secretPayloads: {
        'DBFTPlugin_private_key': 'L1...mock-private-key'
      },
      runtimeImage: dbftDef?.defaultImage || 'neo-cli-plugin',
      configData: mockPluginUserInput,
    });

    const parsedPlugin = JSON.parse(pluginJson);
    expect(parsedPlugin.pluginId).toBe('DBFTPlugin');
    expect(parsedPlugin.config.BlockTxNumber).toBe(1024);
    expect(parsedPlugin.secretPayloads['DBFTPlugin_private_key']).toBe('L1...mock-private-key');

    // 7. Finally, RemotePluginSync constructs the SSH command to push the plugin to the VM
    const sshCommand = buildRemotePluginSyncCommand({
      host: '198.51.100.1',
      user: 'root',
      identityPath: '/app/secrets/operator.key',
      remotePath: '/etc/neonexus/plugins/dbft.json',
      renderedConfig: pluginJson,
    });

    expect(sshCommand).toContain('ssh -i /app/secrets/operator.key');
    expect(sshCommand).toContain('root@198.51.100.1');
    expect(sshCommand).toContain("base64 -d > '/etc/neonexus/plugins/dbft.json'");
    expect(sshCommand).toContain('neonexus-plugin-sync');
  });

  it('successfully simulates the lifecycle of a high-throughput RPC node', () => {
    // 1. User selects RPC Template
    const template = getTemplateById('rpc');
    expect(template?.recommendedEngine).toBe('neo-go');

    // 2. User tweaks NodeSettings to increase peers
    const userSettings = mergeNodeSettings('neo-go', {
      maxPeers: 500,
      rpcEnabled: true,
    });

    // 3. Render artifacts
    const runtime = renderNodeRuntimeArtifacts({
      clientEngine: 'neo-go',
      network: 'mainnet',
      settings: userSettings,
    });

    // neo-go requires a protocol.yml config file generation, unlike neo-cli which just uses flags
    expect(runtime.neoGoConfig).toBeDefined();
    expect(runtime.neoGoConfig).toContain('MaxPeers: 500'); // Validating template merging worked
    expect(runtime.neoGoConfig).toContain('Enabled: true'); // RPC enabled

    // 4. Validate the run command correctly mounts the generated config
    expect(runtime.runCommand).toContain('-v /etc/neonexus/protocol.mainnet.yml:/config/protocol.mainnet.yml');
    expect(runtime.runCommand).toContain('node --config-path');
  });
});
