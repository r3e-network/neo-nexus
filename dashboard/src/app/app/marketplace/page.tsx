import { auth } from '@/auth';
import { prisma } from '@/utils/prisma';
import MarketplaceClient from './MarketplaceClient';

export default async function MarketplacePage() {
  const session = await auth();
  let billingPlan = 'developer'; // default

  if (session?.user?.id && process.env.DATABASE_URL) {
    let orgId = (session.user as any).organizationId;
    
    if (!orgId) {
      const userDb = await prisma.user.findUnique({
        where: { id: session.user.id },
        include: { organization: true }
      });
      if (userDb?.organization) {
        billingPlan = userDb.organization.billingPlan;
      }
    } else {
        const org = await prisma.organization.findUnique({ where: { id: orgId } });
        if (org) billingPlan = org.billingPlan;
    }
  }

  return <MarketplaceClient billingPlan={billingPlan} />;
}
