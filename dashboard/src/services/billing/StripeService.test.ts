import type Stripe from 'stripe';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  organizationUpdate: vi.fn(),
  organizationUpdateMany: vi.fn(),
  normalizeBillingPlan: vi.fn((plan: string | null | undefined) => (
    plan === 'dedicated' || plan === 'growth' ? plan : 'developer'
  )),
}));

vi.mock('@/utils/prisma', () => ({
  prisma: {
    organization: {
      update: mocks.organizationUpdate,
      updateMany: mocks.organizationUpdateMany,
    },
  },
}));

vi.mock('@/server/organization', () => ({
  normalizeBillingPlan: mocks.normalizeBillingPlan,
}));

import { StripeService } from './StripeService';

describe('StripeService', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    process.env.STRIPE_PRICE_ID_GROWTH = 'price_growth';
    process.env.STRIPE_PRICE_ID_DEDICATED = 'price_dedicated';
  });

  it('syncs organization billing plans from customer subscription updates', async () => {
    const listSubscriptions = vi.fn().mockResolvedValue({
      data: [
        {
          status: 'active',
          items: {
            data: [
              {
                price: {
                  id: 'price_dedicated',
                },
              },
            ],
          },
        },
      ],
    });

    vi.spyOn(StripeService as unknown as { stripe: Stripe }, 'stripe', 'get').mockReturnValue({
      subscriptions: {
        list: listSubscriptions,
      },
    } as unknown as Stripe);

    await StripeService.handleWebhook({
      type: 'customer.subscription.updated',
      data: {
        object: {
          customer: 'cus_existing',
        },
      },
    } as unknown as Stripe.Event);

    expect(listSubscriptions).toHaveBeenCalledWith({
      customer: 'cus_existing',
      status: 'all',
      limit: 20,
    });
    expect(mocks.organizationUpdateMany).toHaveBeenCalledWith({
      where: { stripeCustomerId: 'cus_existing' },
      data: { billingPlan: 'dedicated' },
    });
  });

  it('downgrades organizations to developer when Stripe reports no active subscriptions', async () => {
    const listSubscriptions = vi.fn().mockResolvedValue({
      data: [
        {
          status: 'canceled',
          items: {
            data: [
              {
                price: {
                  id: 'price_growth',
                },
              },
            ],
          },
        },
      ],
    });

    vi.spyOn(StripeService as unknown as { stripe: Stripe }, 'stripe', 'get').mockReturnValue({
      subscriptions: {
        list: listSubscriptions,
      },
    } as unknown as Stripe);

    await StripeService.handleWebhook({
      type: 'customer.subscription.deleted',
      data: {
        object: {
          customer: 'cus_existing',
        },
      },
    } as unknown as Stripe.Event);

    expect(mocks.organizationUpdateMany).toHaveBeenCalledWith({
      where: { stripeCustomerId: 'cus_existing' },
      data: { billingPlan: 'developer' },
    });
  });
});
