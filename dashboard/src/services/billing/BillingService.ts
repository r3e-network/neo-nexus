/**
 * Infrastructure Billing & Usage Calculation Service
 */

export class BillingService {
    /**
     * Calculates the projected monthly cost for a node deployment
     */
    static calculateProjectedCost(params: {
        type: string;
        syncMode: string;
        plugins: string[];
    }): number {
        let cost = 0;

        // Base Type Cost
        if (params.type.toLowerCase() === 'dedicated') {
            cost += 99; // Base dedicated server
        }

        // Storage/Sync Cost
        if (params.syncMode.toLowerCase() === 'archive') {
            cost += 50; // Premium SSD storage for 2TB+ archive
        }

        // Plugin Costs
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
