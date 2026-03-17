import type { InfrastructureProvider } from './ProviderCatalog';

export interface DeploymentConfig {
  name: string;
  protocol: 'neo-n3' | 'neo-x';
  network: 'mainnet' | 'testnet' | 'private';
  type: 'shared' | 'dedicated';
  clientEngine: 'neo-go' | 'neo-cli' | 'neo-x-geth';
  provider: InfrastructureProvider;
  region: string;
  syncMode: 'full' | 'archive';
}
