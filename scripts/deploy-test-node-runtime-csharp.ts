import { renderNodeRuntimeArtifacts } from '../dashboard/src/services/settings/RenderedNodeRuntime';
import { buildDefaultNodeSettings, mergeNodeSettings } from '../dashboard/src/services/settings/NodeSettings';
import { execSync } from 'child_process';
import * as fs from 'fs';

// 1. Generate Neo-CLI Config (Using an alpine image with a fake dotnet entrypoint just to prove the docker and plugin system works in theory without hitting a 404 on the image registry)
const settings = mergeNodeSettings('neo-cli', {
  rpcEnabled: true,
  envVars: { 'TEST_VAR': 'hello' },
});

const artifacts = renderNodeRuntimeArtifacts({
  clientEngine: 'neo-cli',
  network: 'mainnet',
  settings
});

// Hack the run script to use a mock image that won't fail to pull
const runScript = artifacts.runScript.replace('neo-project/neo-cli:3.7.4', 'alpine').replace('dotnet neo-cli.dll --rpc', 'tail -f /dev/null');

fs.writeFileSync('/tmp/neonexus-node-sync', runScript);

console.log('Pushing Neo-CLI configuration to VM...');
execSync(`scp -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no /tmp/neonexus-node-sync root@91.99.197.255:/usr/local/bin/neonexus-node-sync`);

console.log('Executing node sync on VM to start neo-cli mock...');
execSync(`ssh -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no root@91.99.197.255 "chmod +x /usr/local/bin/neonexus-node-sync && /usr/local/bin/neonexus-node-sync"`, { stdio: 'inherit' });

console.log('Deployed neo-cli mock!');
