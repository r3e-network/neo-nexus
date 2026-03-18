import { renderNodeRuntimeArtifacts } from '../dashboard/src/services/settings/RenderedNodeRuntime';
import { buildDefaultNodeSettings, mergeNodeSettings } from '../dashboard/src/services/settings/NodeSettings';
import { execSync } from 'child_process';
import * as fs from 'fs';

// 1. Generate Neo-CLI Config (Using the new neonexus/neo-cli:v3.9.2)
const settings = mergeNodeSettings('neo-cli', {
  rpcEnabled: true,
});

const artifacts = renderNodeRuntimeArtifacts({
  clientEngine: 'neo-cli',
  network: 'mainnet',
  settings
});

fs.writeFileSync('/tmp/neonexus-node-sync', artifacts.runScript);

console.log('Pushing latest Neo-CLI (v3.9.2) configuration to VM...');
execSync(`scp -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no /tmp/neonexus-node-sync root@91.99.197.255:/usr/local/bin/neonexus-node-sync`);

console.log('Executing node sync on VM to start the latest neo-cli image...');
execSync(`ssh -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no root@91.99.197.255 "chmod +x /usr/local/bin/neonexus-node-sync && /usr/local/bin/neonexus-node-sync"`, { stdio: 'inherit' });

console.log('Deployed latest neo-cli!');
