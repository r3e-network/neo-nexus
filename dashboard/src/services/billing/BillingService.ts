/**
 * Infrastructure Billing & Usage Calculation Service
 */

import type { InfrastructureProvider } from '@/services/infrastructure/ProviderCatalog';

export class BillingService {
    /**
     * Calculates the projected monthly cost for a node deployment
     */
    static calculateProjectedCost(params: {
        type: string;
        syncMode: string;
        plugins: string[];
        provider?: InfrastructureProvider;
    }): number {
        let cost = 0;
        const provider = params.provider ?? 'hetzner';

        if (params.type.toLowerCase() === 'dedicated') {
            cost += provider === 'hetzner' ? 79 : 99;
        }

        if (params.syncMode.toLowerCase() === 'archive') {
            cost += 50;
        }

        params.plugins.forEach(plugin => {
            switch(plugin) {
                case 'aa-bundler':
                    cost += 49;
                    break;
                case 'tee-oracle':
                    cost += 99;
                    break;
            }
        });

        return cost;
    }
}
