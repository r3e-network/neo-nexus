/**
 * Apache APISIX Integration Service
 * 
 * Manages dynamic routing, consumers (API Keys), and rate limiting 
 * via the APISIX Admin API.
 */

export class ApisixService {
    private static baseUrl = process.env.APISIX_ADMIN_URL;
    private static apiKey = process.env.APISIX_ADMIN_KEY;

    /**
     * Creates a route mapping a public URL path to an upstream target.
     */
    static async createRoute(
        routeKey: string,
        publicUrl: string,
        upstreamHost: string,
        port: number,
        extraPlugins: Record<string, unknown> = {},
    ) {
        const routeId = `endpoint-${routeKey}`;
        const payload = buildRoutePayload(routeKey, publicUrl, upstreamHost, port, extraPlugins);

        return this.sendRequest(`/routes/${routeId}`, 'PUT', payload);
    }

    static async deleteRoute(routeKey: string) {
        const routeId = `endpoint-${routeKey}`;
        return this.sendRequest(`/routes/${routeId}`, 'DELETE');
    }

    /**
     * Creates a Consumer (Tenant) in APISIX and assigns an API Key and Rate Limits.
     */
    static async createConsumer(apiKeyId: string, apiKey: string, plan: 'developer' | 'growth' | 'dedicated') {
        const consumerId = buildConsumerId(apiKeyId);
        const payload = buildConsumerPayload(apiKeyId, apiKey, plan);
        return this.sendRequest(`/consumers/${consumerId}`, 'PUT', payload);
    }

    static async deleteConsumer(apiKeyId: string) {
        const consumerId = buildConsumerId(apiKeyId);
        return this.sendRequest(`/consumers/${consumerId}`, 'DELETE');
    }

    private static async sendRequest(path: string, method: string, data?: Record<string, unknown>) {
        if (!this.baseUrl || !this.apiKey) {
            throw new Error('[APISIX] APISIX_ADMIN_URL and APISIX_ADMIN_KEY must be configured in production.');
        }

        const baseUrl = this.baseUrl;
        const apiKey = this.apiKey;

        try {
            const response = await fetch(`${baseUrl}${path}`, {
                method,
                headers: {
                    'X-API-KEY': apiKey,
                    'Content-Type': 'application/json'
                },
                body: data ? JSON.stringify(data) : undefined
            });

            if (!response.ok) {
                const errorText = await response.text();
                throw new Error(`APISIX Error: ${response.status} ${errorText}`);
            }
            return true;
        } catch (error) {
            console.error(`[APISIX Service] Failed to sync configuration:`, error);
            return false;
        }
    }
}

export function buildConsumerId(apiKeyId: string) {
    return `api-key-${apiKeyId}`;
}

export function buildConsumerPayload(
    apiKeyId: string,
    apiKey: string,
    plan: 'developer' | 'growth' | 'dedicated',
) {
    let rateLimit = 30;
    let burstLimit = 10;

    if (plan === 'growth') {
        rateLimit = 150;
        burstLimit = 50;
    } else if (plan === 'dedicated') {
        rateLimit = 10000;
        burstLimit = 2000;
    }

    return {
        username: buildConsumerId(apiKeyId),
        plugins: {
            "key-auth": {
                key: apiKey
            },
            "limit-req": {
                rate: rateLimit,
                burst: burstLimit,
                rejected_code: 429,
                key_type: "var",
                key: "consumer_name"
            }
        }
    };
}

export function buildRoutePayload(
    routeKey: string,
    publicUrl: string,
    upstreamHost: string,
    port: number,
    extraPlugins: Record<string, unknown> = {},
) {
    const parsedUrl = new URL(publicUrl);
    const normalizedPath = parsedUrl.pathname.endsWith('/')
        ? parsedUrl.pathname.slice(0, -1)
        : parsedUrl.pathname;
    const routePath = normalizedPath === '' ? '/*' : `${normalizedPath}*`;

    const plugins: Record<string, unknown> = {
        "key-auth": {},
        ...extraPlugins,
    };

    return {
        name: routeKey,
        hosts: [parsedUrl.hostname],
        uri: routePath,
        upstream: {
            type: "roundrobin",
            nodes: {
                [`${upstreamHost}:${port}`]: 1
            }
        },
        plugins,
    };
}
