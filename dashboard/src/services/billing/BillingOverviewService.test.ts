import { describe, expect, it } from 'vitest';
import { buildBillingOverview } from './BillingOverviewService';

describe('BillingOverviewService', () => {
  it('maps crypto transactions into a unified history sorted by date', () => {
    const overview = buildBillingOverview({
      cryptoTransactions: [
        {
          id: 7,
          plan: 'dedicated',
          txHash: '0xabc',
          amountAtomic: '3000000000',
          verifiedAt: '2026-03-17T10:00:00.000Z',
        },
        {
          id: 8,
          plan: 'growth',
          txHash: '0xxyz',
          amountAtomic: '1000000000',
          verifiedAt: '2026-03-18T10:00:00.000Z',
        },
      ],
    });

    expect(overview.items).toHaveLength(2);
    // Should be sorted by date descending, so id 8 is first
    expect(overview.items[0]).toMatchObject({
      source: 'crypto',
      title: 'Growth plan',
      amountLabel: '10 GAS',
    });
    expect(overview.items[1]).toMatchObject({
      source: 'crypto',
      title: 'Dedicated plan',
      amountLabel: '30 GAS',
    });
  });
});
