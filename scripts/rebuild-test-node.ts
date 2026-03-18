import { buildNodeBootstrapScript } from '../dashboard/src/services/provisioning/NodeBootstrap';
import * as fs from 'fs';

async function main() {
  const publicKey = fs.readFileSync('/home/neo/.ssh/id_ed25519.pub', 'utf8').trim();
  const token = '***REMOVED***';
  const serverId = 123128624; // 91.99.197.255

  const bootstrapScript = buildNodeBootstrapScript({
    endpointName: 'Production-Consensus-Test',
    protocol: 'neo-n3',
    clientEngine: 'neo-cli',
    network: 'mainnet',
    syncMode: 'full',
    operatorPublicKey: publicKey,
  });

  console.log('Rebuilding server', serverId, 'with our bootstrap script...');
  
  const res = await fetch(`https://api.hetzner.cloud/v1/servers/${serverId}/actions/rebuild`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      image: 'ubuntu-24.04',
      ssh_keys: [108760874], // "jimmy@r3e.network"
      user_data: bootstrapScript
    })
  });

  if (!res.ok) {
    console.error('Failed to rebuild', await res.text());
  } else {
    console.log('Rebuild started! Response:', await res.json());
  }
}

main().catch(console.error);
