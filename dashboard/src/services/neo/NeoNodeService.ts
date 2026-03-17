/**
 * Neo N3 Node Health & Interaction Service
 * 
 * This service is responsible for performing health checks and 
 * directly querying deployed Neo N3 nodes to verify their status.
 */

export class NeoNodeService {
    private static createRequest(url: string, clientEngine: string, method: string, params: unknown[]) {
        return fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                jsonrpc: '2.0',
                id: 1,
                method,
                params
            }),
            signal: AbortSignal.timeout(3000)
        });
    }

    /**
     * Pings the RPC endpoint to check if the node is responsive
     */
    static async checkHealth(url: string, clientEngine = 'neo-go'): Promise<boolean> {
        try {
            const method = clientEngine === 'neo-x-geth' ? 'web3_clientVersion' : 'getversion';
            const response = await this.createRequest(url, clientEngine, method, []);

            if (!response.ok) return false;
            const data = await response.json();
            if (clientEngine === 'neo-x-geth') {
                return typeof data.result === 'string' && data.result.length > 0;
            }

            return !!data.result?.useragent;
        } catch (error) {
            console.error(`[NeoNodeService] Health check failed for ${url}:`, error);
            return false;
        }
    }

    /**
     * Gets the current block height of the node
     */
    static async getBlockCount(url: string, clientEngine = 'neo-go'): Promise<number | null> {
        try {
            const method = clientEngine === 'neo-x-geth' ? 'eth_blockNumber' : 'getblockcount';
            const response = await this.createRequest(url, clientEngine, method, []);

            if (!response.ok) return null;
            const data = await response.json();

            if (clientEngine === 'neo-x-geth' && typeof data.result === 'string') {
                return Number.parseInt(data.result, 16);
            }

            return typeof data.result === 'number' ? data.result : null;
        } catch {
            return null;
        }
    }

    /**
     * Gets the number of connected peers
     */
    static async getPeersCount(url: string, clientEngine = 'neo-go'): Promise<number | null> {
        try {
            const method = clientEngine === 'neo-x-geth' ? 'net_peerCount' : 'getpeers';
            const response = await this.createRequest(url, clientEngine, method, []);

            if (!response.ok) return null;
            const data = await response.json();

            if (clientEngine === 'neo-x-geth' && typeof data.result === 'string') {
                return Number.parseInt(data.result, 16);
            }

            if (data.result?.connected) {
                return data.result.connected.length;
            }
            return null;
        } catch {
            return null;
        }
    }
}
