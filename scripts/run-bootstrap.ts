import { buildNodeBootstrapScript } from '../dashboard/src/services/provisioning/NodeBootstrap';
import { execSync } from 'child_process';
import * as fs from 'fs';

const publicKey = fs.readFileSync('/home/neo/.ssh/id_ed25519.pub', 'utf8').trim();

const bootstrapScript = buildNodeBootstrapScript({
  endpointName: 'Production-RPC-Test',
  protocol: 'neo-n3',
  clientEngine: 'neo-go',
  network: 'mainnet',
  syncMode: 'full',
  operatorPublicKey: publicKey,
});

fs.writeFileSync('/tmp/bootstrap.yaml', bootstrapScript);
console.log('Saved bootstrap script to /tmp/bootstrap.yaml');

// The bootstrap script is a cloud-init yaml file. 
// We can use cloud-init to execute it locally on the server.
const command = `scp -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no /tmp/bootstrap.yaml root@91.99.197.255:/tmp/ && ssh -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no root@91.99.197.255 "cloud-init single -n cc_write_files --file /tmp/bootstrap.yaml && cloud-init single -n cc_runcmd --file /tmp/bootstrap.yaml"`;

console.log('Running cloud-init on remote server...');
try {
  execSync(command, { stdio: 'inherit' });
  console.log('Bootstrap finished!');
} catch (e) {
  console.error('Error running bootstrap');
}
