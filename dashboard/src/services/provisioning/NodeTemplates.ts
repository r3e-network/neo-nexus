import type { SupportedPluginId } from '../plugins/PluginCatalog';

export type NodeTemplateId = 'rpc' | 'consensus' | 'oracle' | 'custom';

export type NodeTemplate = {
  id: NodeTemplateId;
  name: string;
  description: string;
  recommendedEngine: 'neo-go' | 'neo-cli';
  defaultPlugins: SupportedPluginId[];
};

export const NODE_TEMPLATES: NodeTemplate[] = [
  {
    id: 'rpc',
    name: 'RPC Node',
    description: 'Optimized for high-throughput JSON-RPC queries. Includes RpcServer, ApplicationLogs, and TokensTracker.',
    recommendedEngine: 'neo-go',
    defaultPlugins: ['RpcServer', 'ApplicationLogs', 'TokensTracker', 'StateService'],
  },
  {
    id: 'consensus',
    name: 'Consensus Node',
    description: 'Configured to participate in dBFT block validation. Requires a consensus key.',
    recommendedEngine: 'neo-cli',
    defaultPlugins: ['DBFTPlugin'],
  },
  {
    id: 'oracle',
    name: 'Oracle Node',
    description: 'Configured to serve native Neo Oracle requests.',
    recommendedEngine: 'neo-cli',
    defaultPlugins: ['OracleService'],
  },
  {
    id: 'custom',
    name: 'Custom Node',
    description: 'Start with a bare node and configure everything manually.',
    recommendedEngine: 'neo-go',
    defaultPlugins: [],
  },
];

export function getNodeTemplates(): NodeTemplate[] {
  return NODE_TEMPLATES;
}

export function getTemplateById(id: NodeTemplateId): NodeTemplate | undefined {
  return NODE_TEMPLATES.find(t => t.id === id);
}
