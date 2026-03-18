import BillingClient from './BillingClient';
import { getCurrentUserContext } from '@/server/organization';
import { getPublicCryptoBillingConfig } from '@/services/billing/CryptoBillingService';
import { prisma } from '@/utils/prisma';
import { buildBillingOverview } from '@/services/billing/BillingOverviewService';

export const dynamic = 'force-dynamic';

export default async function BillingPage() {
  const userContext = await getCurrentUserContext();
  const billingPlan = userContext?.billingPlan ?? 'developer';
  const cryptoBillingConfig = getPublicCryptoBillingConfig();
  let billingOverview = buildBillingOverview({
    cryptoTransactions: [],
  });

  if (process.env.DATABASE_URL && userContext?.organizationId) {
    const organization = await prisma.organization.findUnique({
      where: { id: userContext.organizationId },
      select: {
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

    billingOverview = buildBillingOverview({
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
      billingOverview={billingOverview}
    />
  );
}
