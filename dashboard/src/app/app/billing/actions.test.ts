import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  requireCurrentOrganizationContext: vi.fn(),
  organizationFindUnique: vi.fn(),
  createCheckoutSession: vi.fn(),
  createBillingPortalSession: vi.fn(),
  redirect: vi.fn(),
}));

vi.mock('@/server/organization', () => ({
  assertDatabaseConfigured: vi.fn(),
  requireCurrentOrganizationContext: mocks.requireCurrentOrganizationContext,
}));

vi.mock('@/server/errors', () => ({
  getErrorMessage: vi.fn((error: unknown) => error instanceof Error ? error.message : 'Unknown error'),
}));

vi.mock('@/utils/prisma', () => ({
  prisma: {
    organization: {
      findUnique: mocks.organizationFindUnique,
    },
  },
}));

vi.mock('@/services/billing/StripeService', () => ({
  StripeService: {
    createCheckoutSession: mocks.createCheckoutSession,
    createBillingPortalSession: mocks.createBillingPortalSession,
  },
}));

vi.mock('@/services/billing/CryptoBillingService', () => ({
  verifyCryptoTransferOnChain: vi.fn(),
}));

vi.mock('next/navigation', () => ({
  redirect: mocks.redirect,
}));

import { upgradePlanAction } from './actions';

describe('billing upgrade actions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    process.env.NEXT_PUBLIC_APP_URL = 'https://app.test';
    mocks.requireCurrentOrganizationContext.mockResolvedValue({
      userId: 'user_123',
      organizationId: 'org_123',
      billingPlan: 'growth',
      role: 'member',
    });
    mocks.redirect.mockImplementation((url: string) => {
      throw new Error(`REDIRECT:${url}`);
    });
  });

  it('sends existing Stripe customers to the billing portal instead of starting a second checkout', async () => {
    mocks.organizationFindUnique.mockResolvedValue({
      stripeCustomerId: 'cus_123',
    });
    mocks.createBillingPortalSession.mockResolvedValue({
      url: 'https://billing.stripe.test/session',
    });

    await expect(upgradePlanAction('dedicated')).rejects.toThrow(
      'REDIRECT:https://billing.stripe.test/session',
    );

    expect(mocks.organizationFindUnique).toHaveBeenCalledWith({
      where: { id: 'org_123' },
      select: { stripeCustomerId: true },
    });
    expect(mocks.createBillingPortalSession).toHaveBeenCalledWith(
      'cus_123',
      'https://app.test/app/billing',
    );
    expect(mocks.createCheckoutSession).not.toHaveBeenCalled();
    expect(mocks.redirect).toHaveBeenCalledWith('https://billing.stripe.test/session');
  });

  it('starts checkout for first-time Stripe subscriptions', async () => {
    mocks.organizationFindUnique.mockResolvedValue({
      stripeCustomerId: null,
    });
    mocks.createCheckoutSession.mockResolvedValue({
      url: 'https://checkout.stripe.test/session',
    });

    await expect(upgradePlanAction('growth')).rejects.toThrow(
      'REDIRECT:https://checkout.stripe.test/session',
    );

    expect(mocks.createCheckoutSession).toHaveBeenCalledWith(
      'org_123',
      'growth',
      'https://app.test/app/billing?success=true',
      'https://app.test/app/billing?canceled=true',
    );
    expect(mocks.createBillingPortalSession).not.toHaveBeenCalled();
    expect(mocks.redirect).toHaveBeenCalledWith('https://checkout.stripe.test/session');
  });
});
