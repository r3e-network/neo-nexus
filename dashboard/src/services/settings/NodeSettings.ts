export type NodeSettings = {
  maxPeers: number;
  rpcEnabled: boolean;
  websocketEnabled: boolean;
  graphqlEnabled: boolean;
  cacheMb: number | null;
};

export function buildDefaultNodeSettings(clientEngine: string): NodeSettings {
  if (clientEngine === 'neo-x-geth') {
    return {
      maxPeers: 50,
      rpcEnabled: true,
      websocketEnabled: true,
      graphqlEnabled: false,
      cacheMb: 4096,
    };
  }

  return {
    maxPeers: 100,
    rpcEnabled: true,
    websocketEnabled: false,
    graphqlEnabled: false,
    cacheMb: null,
  };
}

export function mergeNodeSettings(
  clientEngine: string,
  rawSettings: unknown,
): NodeSettings {
  const defaults = buildDefaultNodeSettings(clientEngine);
  if (!rawSettings || typeof rawSettings !== 'object') {
    return defaults;
  }

  const settings = rawSettings as Partial<NodeSettings>;
  return {
    maxPeers: typeof settings.maxPeers === 'number' ? settings.maxPeers : defaults.maxPeers,
    rpcEnabled: typeof settings.rpcEnabled === 'boolean' ? settings.rpcEnabled : defaults.rpcEnabled,
    websocketEnabled: typeof settings.websocketEnabled === 'boolean' ? settings.websocketEnabled : defaults.websocketEnabled,
    graphqlEnabled: typeof settings.graphqlEnabled === 'boolean' ? settings.graphqlEnabled : defaults.graphqlEnabled,
    cacheMb: typeof settings.cacheMb === 'number' ? settings.cacheMb : defaults.cacheMb,
  };
}
