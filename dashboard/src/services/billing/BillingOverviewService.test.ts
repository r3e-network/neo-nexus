import { describe, expect, it } from 'vitest';
import { buildBillingOverview } from './BillingOverviewService';

describe('BillingOverviewService', () => {
  it('combines stripe invoices and crypto transactions into a unified history sorted by date', () => {
    const overview = buildBillingOverview({
      stripeInvoices: [
        {
          id: 'in_123',
          amountPaidCents: 4900,
          currency: 'usd',
          status: 'paid',
          hostedInvoiceUrl: 'https://stripe.test/in_123',
          createdAt: '2026-03-16T10:00:00.000Z',
          description: 'Growth plan',
        },
      ],
      cryptoTransactions: [
        {
          id: 7,
          plan: 'dedicated',
          txHash: '0xabc',
          amountAtomic: '3000000000',
          verifiedAt: '2026-03-17T10:00:00.000Z',
        },
      ],
    });

    expect(overview.items).toHaveLength(2);
    expect(overview.items[0]).toMatchObject({
      source: 'crypto',
      title: 'Dedicated plan',
      amountLabel: '30 GAS',
    });
    expect(overview.items[1]).toMatchObject({
      source: 'stripe',
      title: 'Growth plan',
    });
  });

  it('builds a card summary when stripe card data exists', () => {
    const overview = buildBillingOverview({
      stripeCard: {
        brand: 'visa',
        last4: '4242',
        expMonth: 12,
        expYear: 2030,
      },
      stripeInvoices: [],
      cryptoTransactions: [],
    });

    expect(overview.cardSummary).toBe('VISA ending in 4242');
    expect(overview.cardExpiry).toBe('12/2030');
  });
});
