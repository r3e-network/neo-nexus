/**
 * Neo N3 Node Health & Interaction Service
 * 
 * This service is responsible for performing health checks and 
 * directly querying deployed Neo N3 nodes to verify their status.
 */

export class NeoNodeService {
    /**
     * Pings the RPC endpoint to check if the node is responsive
     */
    static async checkHealth(url: string): Promise<boolean> {
        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    id: 1,
                    method: 'getversion',
                    params: []
                }),
                // Short timeout to not block UI rendering
                signal: AbortSignal.timeout(3000)
            });

            if (!response.ok) return false;
            const data = await response.json();
            return !!data.result?.useragent;
        } catch (error) {
            console.error(`[NeoNodeService] Health check failed for ${url}:`, error);
            return false;
        }
    }

    /**
     * Gets the current block height of the node
     */
    static async getBlockCount(url: string): Promise<number | null> {
        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    id: 1,
                    method: 'getblockcount',
                    params: []
                }),
                signal: AbortSignal.timeout(3000)
            });

            if (!response.ok) return null;
            const data = await response.json();
            return typeof data.result === 'number' ? data.result : null;
        } catch {
            return null;
        }
    }

    /**
     * Gets the number of connected peers
     */
    static async getPeersCount(url: string): Promise<number | null> {
        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    id: 1,
                    method: 'getpeers',
                    params: []
                }),
                signal: AbortSignal.timeout(3000)
            });

            if (!response.ok) return null;
            const data = await response.json();
            if (data.result?.connected) {
                return data.result.connected.length;
            }
            return null;
        } catch {
            return null;
        }
    }
}
