import BillingClient from './BillingClient';
import { getCurrentUserContext } from '@/server/organization';
import { getPublicCryptoBillingConfig } from '@/services/billing/CryptoBillingService';
import { prisma } from '@/utils/prisma';
import { buildBillingOverview } from '@/services/billing/BillingOverviewService';
import { StripeService } from '@/services/billing/StripeService';

export const dynamic = 'force-dynamic';

export default async function BillingPage() {
  const userContext = await getCurrentUserContext();
  const billingPlan = userContext?.billingPlan ?? 'developer';
  const cryptoBillingConfig = getPublicCryptoBillingConfig();
  let hasStripeCustomer = false;
  let billingOverview = buildBillingOverview({
    stripeInvoices: [],
    cryptoTransactions: [],
  });

  if (process.env.DATABASE_URL && userContext?.organizationId) {
    const organization = await prisma.organization.findUnique({
      where: { id: userContext.organizationId },
      select: {
        stripeCustomerId: true,
        billingTransactions: {
          orderBy: { verifiedAt: 'desc' },
          take: 10,
          select: {
            id: true,
            plan: true,
            txHash: true,
            amountAtomic: true,
            verifiedAt: true,
          },
        },
      },
    });
    hasStripeCustomer = Boolean(organization?.stripeCustomerId);

    const stripeInvoices = hasStripeCustomer && process.env.STRIPE_SECRET_KEY
      ? await StripeService.listRecentInvoices(organization!.stripeCustomerId!)
      : [];
    const stripeCard = hasStripeCustomer && process.env.STRIPE_SECRET_KEY
      ? await StripeService.getCardSummary(organization!.stripeCustomerId!)
      : null;

    billingOverview = buildBillingOverview({
      stripeInvoices,
      stripeCard,
      cryptoTransactions: (organization?.billingTransactions ?? []).map((transaction) => ({
        id: transaction.id,
        plan: transaction.plan,
        txHash: transaction.txHash,
        amountAtomic: transaction.amountAtomic.toString(),
        verifiedAt: transaction.verifiedAt.toISOString(),
      })),
    });
  }

  return (
    <BillingClient
      billingPlan={billingPlan}
      cryptoBillingConfig={cryptoBillingConfig}
      hasStripeCustomer={hasStripeCustomer}
      billingOverview={billingOverview}
    />
  );
}
