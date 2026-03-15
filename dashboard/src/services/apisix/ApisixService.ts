/**
 * Apache APISIX Integration Service
 * 
 * Manages dynamic routing, consumers (API Keys), and rate limiting 
 * via the APISIX Admin API.
 */

export class ApisixService {
    private static baseUrl = process.env.APISIX_ADMIN_URL || 'http://apisix-admin:9180/apisix/admin';
    private static apiKey = process.env.APISIX_ADMIN_KEY || 'edd1c9f034335f136f87ad84b625c8f1'; // Default APISIX dev key

    /**
     * Creates a route mapping a public URL path to an internal Kubernetes Service.
     */
    static async createRoute(endpointId: string, internalHost: string, port: number) {
        const routeId = `endpoint-${endpointId}`;
        const payload = {
            uri: `/v1/${endpointId}/*`,
            upstream: {
                type: "roundrobin",
                nodes: {
                    [`${internalHost}:${port}`]: 1
                }
            },
            plugins: {
                "key-auth": {}, // Enforce API Key
                "limit-req": {  // Base protection against DDoS
                    rate: 100,
                    burst: 50,
                    rejected_code: 429,
                    key_type: "var",
                    key: "remote_addr"
                }
            }
        };

        return this.sendRequest(`/routes/${routeId}`, 'PUT', payload);
    }

    /**
     * Creates a Consumer (Tenant) in APISIX and assigns an API Key and Rate Limits.
     */
    static async createConsumer(organizationId: string, apiKey: string, plan: 'developer' | 'growth' | 'dedicated') {
        let rateLimit = 30; // requests per second
        let burstLimit = 10;
        
        if (plan === 'growth') {
            rateLimit = 150;
            burstLimit = 50;
        } else if (plan === 'dedicated') {
            rateLimit = 10000; // Effectively unlimited
            burstLimit = 2000;
        }

        const payload = {
            username: `org-${organizationId}`,
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

        return this.sendRequest(`/consumers`, 'PUT', payload);
    }

    private static async sendRequest(path: string, method: string, data: any) {
        if (!process.env.APISIX_ADMIN_URL || !process.env.APISIX_ADMIN_KEY) {
            throw new Error('[APISIX] APISIX_ADMIN_URL and APISIX_ADMIN_KEY must be configured in production.');
        }

        try {
            const response = await fetch(`${process.env.APISIX_ADMIN_URL}${path}`, {
                method,
                headers: {
                    'X-API-KEY': process.env.APISIX_ADMIN_KEY,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(data)
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
