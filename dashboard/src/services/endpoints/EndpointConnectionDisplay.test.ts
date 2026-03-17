import { describe, expect, it } from 'vitest';
import {
  deriveEndpointConnectionDisplay,
  deriveEndpointConnectionUrls,
} from './EndpointConnectionDisplay';

describe('EndpointConnectionDisplay', () => {
  it('preserves an explicit websocket URL when one is provided', () => {
    const urls = deriveEndpointConnectionUrls({
      url: 'https://rpc.example.com/v1',
      wssUrl: 'wss://ws.example.com/custom',
    });

    expect(urls.httpsUrl).toBe('https://rpc.example.com/v1');
    expect(urls.wssUrl).toBe('wss://ws.example.com/custom');
  });

  it('derives a websocket URL from the https endpoint when needed', () => {
    const urls = deriveEndpointConnectionUrls({
      url: 'https://rpc.example.com/v1',
      wssUrl: null,
    });

    expect(urls.wssUrl).toBe('wss://rpc.example.com/ws');
  });

  it('returns null connection URLs when routing is not configured yet', () => {
    const urls = deriveEndpointConnectionUrls({
      url: null,
      wssUrl: null,
    });

    expect(urls.httpsUrl).toBeNull();
    expect(urls.wssUrl).toBeNull();
  });

  it('marks provisioning endpoints as planned rather than live', () => {
    const display = deriveEndpointConnectionDisplay({
      status: 'Provisioning',
      url: 'https://node-fsn1-42.neonexus.cloud/v1',
      wssUrl: null,
    });

    expect(display.readiness).toBe('planned');
    expect(display.label).toBe('Planned route');
  });

  it('marks stopped endpoints as offline', () => {
    const display = deriveEndpointConnectionDisplay({
      status: 'Stopped',
      url: 'https://node-fsn1-42.neonexus.cloud/v1',
      wssUrl: null,
    });

    expect(display.readiness).toBe('offline');
    expect(display.label).toBe('Route offline');
  });
});
