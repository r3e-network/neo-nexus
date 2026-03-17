import { provisionDedicatedNode } from '../dashboard/src/services/provisioning/VmProvisioner';
import type { DeploymentConfig } from '../dashboard/src/services/infrastructure/DeploymentConfig';
import * as fs from 'fs';

const testConfig: DeploymentConfig = {
  name: 'neonexus-test-node',
  protocol: 'neo-n3',
  clientEngine: 'neo-go',
  network: 'testnet',
  syncMode: 'full',
  provider: 'hetzner',
  region: 'nbg1',
};

async function main() {
  const publicKey = fs.readFileSync('/home/neo/.ssh/id_ed25519.pub', 'utf8').trim();
  console.log('Using Public Key:', publicKey);

  const env = { 
    NEO_NEXUS_HETZNER: 'FKcIzNxZvSSVyYQP9rIHNOvI9t71eewvEVXSFwuIIoRgrPxd8If34grKOfPGNDrF',
    VM_OPERATOR_PUBLIC_KEY: publicKey
  };
  
  console.log('Provisioning new dedicated node on Hetzner...');
  try {
    const result = await provisionDedicatedNode(testConfig, { env });
    console.log('Provisioning successful!');
    console.log(result);
  } catch (err) {
    console.error('Provisioning failed:', err);
  }
}

main().catch(console.error);
