import { provisionDedicatedNode } from '../dashboard/src/services/provisioning/VmProvisioner';
import type { DeploymentConfig } from '../dashboard/src/services/infrastructure/DeploymentConfig';

const testConfig: DeploymentConfig = {
  name: 'test-node',
  protocol: 'neo-n3',
  clientEngine: 'neo-go',
  network: 'testnet',
  syncMode: 'full',
  provider: 'hetzner',
  region: 'nbg1',
};

async function main() {
  const env = { NEO_NEXUS_HETZNER: 'FKcIzNxZvSSVyYQP9rIHNOvI9t71eewvEVXSFwuIIoRgrPxd8If34grKOfPGNDrF' };
  
  console.log('Testing Hetzner API Token validity by attempting a dry run or fetching existing servers...');
  const response = await fetch('https://api.hetzner.cloud/v1/servers', {
    headers: { 'Authorization': `Bearer ${env.NEO_NEXUS_HETZNER}` }
  });
  
  if (!response.ok) {
    console.error('Failed to fetch servers', await response.text());
    return;
  }
  
  const data = await response.json();
  const server = data.servers.find(s => s.name === 'ubuntu-16gb-nbg1-1' || s.public_net.ipv4.ip === '91.99.197.255');
  
  if (server) {
    console.log('Successfully found user server:', server.name);
    console.log('IP:', server.public_net.ipv4.ip);
    console.log('Status:', server.status);
    console.log('Datacenter:', server.datacenter.name);
    console.log('Hetzner API validation successful!');
  } else {
    console.log('Could not find the specific server, but API call succeeded. Found servers:', data.servers.map(s => s.name));
  }
}

main().catch(console.error);
