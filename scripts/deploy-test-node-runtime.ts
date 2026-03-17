import { renderNodeRuntimeArtifacts } from '../dashboard/src/services/settings/RenderedNodeRuntime';
import { buildDefaultNodeSettings } from '../dashboard/src/services/settings/NodeSettings';
import { execSync } from 'child_process';
import * as fs from 'fs';

const settings = buildDefaultNodeSettings('neo-go');
const artifacts = renderNodeRuntimeArtifacts({
  clientEngine: 'neo-go',
  network: 'mainnet',
  settings
});

fs.writeFileSync('/tmp/protocol.mainnet.yml', artifacts.neoGoConfig!);
fs.writeFileSync('/tmp/neonexus-node-sync', artifacts.runScript);

console.log('Pushing configuration to VM...');
execSync(`scp -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no /tmp/protocol.mainnet.yml root@91.99.197.255:/etc/neonexus/protocol.mainnet.yml`);
execSync(`scp -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no /tmp/neonexus-node-sync root@91.99.197.255:/usr/local/bin/neonexus-node-sync`);
execSync(`ssh -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no root@91.99.197.255 "chmod +x /usr/local/bin/neonexus-node-sync && /usr/local/bin/neonexus-node-sync"`, { stdio: 'inherit' });

console.log('Deployed neo-go!');
