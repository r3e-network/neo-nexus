type EndpointType = 'shared' | 'dedicated';
type EndpointProtocol = 'neo-n3' | 'neo-x';
type EndpointNetworkKey = 'mainnet' | 'testnet' | 'private';

export function getEndpointRootDomain(rootDomain = process.env.ENDPOINT_ROOT_DOMAIN): string {
  const normalized = rootDomain?.trim().toLowerCase();
  return normalized || 'neonexus.cloud';
}

export function buildPlannedEndpointAddress(input: {
  type: EndpointType;
  protocol: EndpointProtocol;
  networkKey: EndpointNetworkKey;
  routeKey: string;
  region?: string | null;
  rootDomain?: string;
}) {
  const rootDomain = getEndpointRootDomain(input.rootDomain);

  if (input.type === 'shared') {
    const httpsUrl = `https://${input.networkKey}.${rootDomain}/v1/${input.routeKey}`;

    return {
      httpsUrl,
      wssUrl: input.protocol === 'neo-x'
        ? httpsUrl.replace(/^https:\/\//, 'wss://').replace('/v1/', '/ws/')
        : null,
    };
  }

  const region = input.region?.trim().toLowerCase() || 'global';
  const httpsUrl = `https://node-${region}-${input.routeKey}.${rootDomain}/v1`;

  return {
    httpsUrl,
    wssUrl: input.protocol === 'neo-x'
      ? httpsUrl.replace(/^https:\/\//, 'wss://').replace(/\/v1$/, '/ws')
      : null,
  };
}
