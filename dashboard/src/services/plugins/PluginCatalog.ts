export type SupportedPluginId = 'tee-oracle' | 'aa-bundler' | 'tee-mempool';

export type PluginDefinition = {
  id: SupportedPluginId;
  name: string;
  description: string;
  badge: string;
  requiresPrivateKey: boolean;
  category: 'Security' | 'Execution';
  defaultImage: string;
};

const PLUGINS: PluginDefinition[] = [
  {
    id: 'tee-oracle',
    name: 'TEE Privacy Oracle',
    description: 'Phala CVM instance securely bound to this node.',
    badge: 'Privacy',
    requiresPrivateKey: true,
    category: 'Security',
    defaultImage: 'ghcr.io/neonexus/tee-oracle:latest',
  },
  {
    id: 'aa-bundler',
    name: 'Account Abstraction Bundler',
    description: 'Native AA relay services for smart-account transaction bundling.',
    badge: 'AA',
    requiresPrivateKey: true,
    category: 'Execution',
    defaultImage: 'ghcr.io/neonexus/aa-bundler:latest',
  },
  {
    id: 'tee-mempool',
    name: 'TEE Protected Mempool',
    description: 'Protected pre-confirmation transaction routing for latency-sensitive workloads.',
    badge: 'TEE',
    requiresPrivateKey: true,
    category: 'Security',
    defaultImage: 'ghcr.io/neonexus/tee-mempool:latest',
  },
];

export function listSupportedPlugins(): PluginDefinition[] {
  return PLUGINS;
}

export function isSupportedPlugin(pluginId: string): pluginId is SupportedPluginId {
  return PLUGINS.some((plugin) => plugin.id === pluginId);
}

export function getPluginDefinition(pluginId: string): PluginDefinition | null {
  return PLUGINS.find((plugin) => plugin.id === pluginId) ?? null;
}
