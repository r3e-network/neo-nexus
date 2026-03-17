import type { NodeSettings } from './NodeSettings';

type ClientEngine = 'neo-go' | 'neo-cli' | 'neo-x-geth';
type Network = 'mainnet' | 'testnet' | 'private';
type SupportedNodeSettingKey = keyof NodeSettings;

export const REMOTE_NODE_SETTINGS_PATH = '/etc/neonexus/node-settings.json';
export const REMOTE_NODE_SYNC_PATH = '/usr/local/bin/neonexus-node-sync';
export const REMOTE_NEO_GO_CONFIG_PATH = '/etc/neonexus/protocol.mainnet.yml';

const SUPPORTED_NODE_SETTING_KEYS: Record<ClientEngine, SupportedNodeSettingKey[]> = {
  'neo-go': ['maxPeers', 'rpcEnabled'],
  'neo-cli': ['rpcEnabled'],
  'neo-x-geth': ['maxPeers', 'rpcEnabled', 'websocketEnabled', 'graphqlEnabled', 'cacheMb'],
};

function buildNeoGoConfig(network: Network, settings: NodeSettings): string {
  const isMainnet = network === 'mainnet';

  return [
    'ProtocolConfiguration:',
    `  Magic: ${isMainnet ? 860833102 : 894710606}`,
    '  MaxTraceableBlocks: 2102400',
    '  InitialGasDistribution: 52000000',
    '  MaxTransactionsPerBlock: 512',
    '  MaxValidUntilBlockIncrement: 5760',
    '  SeedList:',
    `    - ${isMainnet ? 'seed1.neo.org:10333' : 'seed1t5.neo.org:20333'}`,
    `    - ${isMainnet ? 'seed2.neo.org:10333' : 'seed2t5.neo.org:20333'}`,
    `    - ${isMainnet ? 'seed3.neo.org:10333' : 'seed3t5.neo.org:20333'}`,
    'ApplicationConfiguration:',
    '  DataDirectoryPath: "/data"',
    '  P2P:',
    '    Port: 10333',
    `    MaxPeers: ${settings.maxPeers}`,
    '  RPC:',
    `    Enabled: ${settings.rpcEnabled ? 'true' : 'false'}`,
    '    Port: 10332',
    '    MaxGasInvoke: 10',
    '  Prometheus:',
    '    Enabled: true',
    '    Port: 2112',
  ].join('\n');
}

function buildNeoGoRunCommand(): string {
  return [
    'docker run -d --name neonexus-node --restart unless-stopped',
    '-p 10332:10332 -p 10333:10333 -p 2112:2112',
    '-v /var/lib/neonexus:/data',
    `-v ${REMOTE_NEO_GO_CONFIG_PATH}:/config/protocol.mainnet.yml`,
    'nspccdev/neo-go:0.106.0',
    'node --config-path /config --relative-path',
  ].join(' ');
}

function buildNeoCliRunCommand(settings: NodeSettings): string {
  const publishedPorts = ['-p 10333:10333'];
  if (settings.rpcEnabled) {
    publishedPorts.unshift('-p 10332:10332');
  }

  return [
    'docker run -d --name neonexus-node --restart unless-stopped',
    ...publishedPorts,
    '-v /var/lib/neonexus:/data',
    'neo-project/neo-cli:3.7.4',
    settings.rpcEnabled ? 'dotnet neo-cli.dll --rpc' : 'dotnet neo-cli.dll',
  ].join(' ');
}

function buildNeoXRunCommand(settings: NodeSettings): string {
  const publishedPorts = ['-p 30303:30303'];
  const command = ['geth', '--datadir /data', `--maxpeers ${settings.maxPeers}`];

  if (settings.rpcEnabled) {
    publishedPorts.unshift('-p 8545:8545');
    command.push('--http', '--http.addr 0.0.0.0', '--http.port 8545');
    if (settings.graphqlEnabled) {
      command.push('--graphql');
    }
  }

  if (settings.websocketEnabled) {
    publishedPorts.push('-p 8546:8546');
    command.push('--ws', '--ws.addr 0.0.0.0', '--ws.port 8546');
  }

  if (typeof settings.cacheMb === 'number' && settings.cacheMb > 0) {
    command.push(`--cache ${settings.cacheMb}`);
  }

  return [
    'docker run -d --name neonexus-node --restart unless-stopped',
    ...publishedPorts,
    '-v /var/lib/neonexus:/data',
    'neofoundation/neo-x-geth:latest',
    command.join(' '),
  ].join(' ');
}

export function getSupportedNodeSettingKeys(clientEngine: ClientEngine): SupportedNodeSettingKey[] {
  return SUPPORTED_NODE_SETTING_KEYS[clientEngine];
}

export function renderNodeRuntimeArtifacts(input: {
  clientEngine: ClientEngine;
  network: Network;
  settings: NodeSettings;
}) {
  let neoGoConfig: string | null = null;
  let runCommand: string;

  if (input.clientEngine === 'neo-go') {
    neoGoConfig = buildNeoGoConfig(input.network, input.settings);
    runCommand = buildNeoGoRunCommand();
  } else if (input.clientEngine === 'neo-cli') {
    runCommand = buildNeoCliRunCommand(input.settings);
  } else {
    runCommand = buildNeoXRunCommand(input.settings);
  }

  const runScript = [
    '#!/usr/bin/env bash',
    'set -euo pipefail',
    'docker rm -f neonexus-node >/dev/null 2>&1 || true',
    runCommand,
  ].join('\n');

  return {
    neoGoConfig,
    runCommand,
    runScript,
    supportedSettingKeys: getSupportedNodeSettingKeys(input.clientEngine),
  };
}
