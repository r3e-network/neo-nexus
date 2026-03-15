import { prisma } from '@/utils/prisma';
import SecurityClient, { ApiKeyType } from './SecurityClient';
import { getCurrentUserContext } from '@/server/organization';

export const dynamic = 'force-dynamic';

export default async function SecurityPage() {
  let keys: ApiKeyType[] = [];
  let billingPlan = 'developer';

  if (process.env.DATABASE_URL) {
    try {
      const userContext = await getCurrentUserContext();
      const orgId = userContext?.organizationId ?? null;

      if (userContext) {
        billingPlan = userContext.billingPlan;
      }

      if (orgId) {
        const data = await prisma.apiKey.findMany({
          where: { organizationId: orgId },
          orderBy: { createdAt: 'asc' },
        });
        if (data && data.length > 0) {
          keys = data.map((key) => ({
            id: key.id,
            name: key.name,
            keyHash: key.keyHash,
            createdAt: key.createdAt,
            isActive: key.isActive,
          }));
        }
      }
    } catch (error) {
      console.warn('Failed to fetch API keys from DB', error);
    }
  }

  return <SecurityClient initialKeys={keys} billingPlan={billingPlan} />;
}
