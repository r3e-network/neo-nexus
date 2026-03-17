type EndpointConnectionInput = {
  status?: string | null;
  url?: string | null;
  wssUrl?: string | null;
};

function normalizeUrl(value: string | null | undefined): string | null {
  if (!value) {
    return null;
  }

  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

export function deriveEndpointConnectionUrls(input: EndpointConnectionInput) {
  const httpsUrl = normalizeUrl(input.url);
  const explicitWssUrl = normalizeUrl(input.wssUrl);

  if (explicitWssUrl) {
    return {
      httpsUrl,
      wssUrl: explicitWssUrl,
    };
  }

  if (!httpsUrl) {
    return {
      httpsUrl: null,
      wssUrl: null,
    };
  }

  return {
    httpsUrl,
    wssUrl: httpsUrl
      .replace(/^https:\/\//, 'wss://')
      .replace(/^http:\/\//, 'ws://')
      .replace(/\/v1$/, '/ws'),
  };
}

export function deriveEndpointConnectionDisplay(input: EndpointConnectionInput) {
  const urls = deriveEndpointConnectionUrls(input);
  const normalizedStatus = input.status?.trim().toLowerCase() ?? null;

  if (!urls.httpsUrl) {
    return {
      ...urls,
      readiness: 'unavailable' as const,
      label: 'Route unavailable',
    };
  }

  if (normalizedStatus === 'active') {
    return {
      ...urls,
      readiness: 'live' as const,
      label: 'Live route',
    };
  }

  if (normalizedStatus === 'provisioning' || normalizedStatus === 'syncing') {
    return {
      ...urls,
      readiness: 'planned' as const,
      label: 'Planned route',
    };
  }

  return {
    ...urls,
    readiness: 'offline' as const,
    label: 'Route offline',
  };
}
