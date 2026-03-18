import { renderNodeRuntimeArtifacts } from '../dashboard/src/services/settings/RenderedNodeRuntime';
import { buildDefaultNodeSettings, mergeNodeSettings } from '../dashboard/src/services/settings/NodeSettings';
import { execSync } from 'child_process';
import * as fs from 'fs';

const settings = mergeNodeSettings('neo-go', {
  rpcEnabled: true,
});

const artifacts = renderNodeRuntimeArtifacts({
  clientEngine: 'neo-go',
  network: 'testnet', // Testing TESTNET explicitly
  settings
});

fs.writeFileSync('/tmp/protocol.testnet.yml', artifacts.neoGoConfig!);
fs.writeFileSync('/tmp/neonexus-node-sync', artifacts.runScript);

console.log('Pushing Testnet configuration to VM...');
execSync(`scp -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no /tmp/protocol.testnet.yml root@91.99.197.255:/etc/neonexus/protocol.mainnet.yml`); // Using mainnet volume mount to override it with testnet config for simulation
execSync(`scp -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no /tmp/neonexus-node-sync root@91.99.197.255:/usr/local/bin/neonexus-node-sync`);

console.log('Executing node sync on VM to start neo-go on Testnet...');
execSync(`ssh -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no root@91.99.197.255 "chmod +x /usr/local/bin/neonexus-node-sync && /usr/local/bin/neonexus-node-sync"`, { stdio: 'inherit' });

console.log('Deployed neo-go on Testnet!');
