import { describe, expect, it } from 'vitest';
import {
  buildConsumerId,
  buildConsumerPayload,
  buildRoutePayload,
} from './ApisixService';

describe('ApisixService', () => {
  it('builds host-based dedicated routes from the public URL', () => {
    const payload = buildRoutePayload(
      'endpoint-1',
      'https://node-fsn1-abc.neonexus.cloud/v1',
      '203.0.113.10',
      10332,
    );

    expect(payload).toMatchObject({
      hosts: ['node-fsn1-abc.neonexus.cloud'],
      uri: '/v1*',
    });
    expect(payload.upstream.nodes['203.0.113.10:10332']).toBe(1);
  });

  it('builds host-and-path shared routes from the public URL', () => {
    const payload = buildRoutePayload(
      'endpoint-2',
      'https://mainnet.neonexus.cloud/v1/r4nd0m',
      'shared-upstream.internal',
      10332,
    );

    expect(payload).toMatchObject({
      hosts: ['mainnet.neonexus.cloud'],
      uri: '/v1/r4nd0m*',
    });
    expect(payload.upstream.nodes['shared-upstream.internal:10332']).toBe(1);
  });

  it('merges route security plugins into the route payload', () => {
    const payload = buildRoutePayload(
      'endpoint-3',
      'https://node-fsn1-42.neonexus.cloud/v1',
      '203.0.113.10',
      10332,
      {
        'ip-restriction': {
          whitelist: ['203.0.113.10'],
        },
      },
    );

    expect(payload.plugins['key-auth']).toBeDefined();
    expect(payload.plugins['ip-restriction']).toMatchObject({
      whitelist: ['203.0.113.10'],
    });
  });

  it('does not apply a static route-level limit-req policy that overrides consumer plan limits', () => {
    const payload = buildRoutePayload(
      'endpoint-4',
      'https://node-fsn1-99.neonexus.cloud/v1',
      '203.0.113.10',
      10332,
    );

    expect(payload.plugins['key-auth']).toBeDefined();
    expect(payload.plugins['limit-req']).toBeUndefined();
  });

  it('builds a unique consumer id per api key', () => {
    expect(buildConsumerId('key-123')).toBe('api-key-key-123');
    expect(buildConsumerId('key-456')).not.toBe(buildConsumerId('key-123'));
  });

  it('builds a consumer payload with the submitted api key, consumer username, and plan limits', () => {
    const payload = buildConsumerPayload('key-123', 'nk_live_example', 'growth');

    expect(payload).toMatchObject({
      username: 'api-key-key-123',
      plugins: {
        'key-auth': {
          key: 'nk_live_example',
        },
        'limit-req': {
          rate: 150,
          burst: 50,
          key: 'consumer_name',
        },
      },
    });
  });
});
