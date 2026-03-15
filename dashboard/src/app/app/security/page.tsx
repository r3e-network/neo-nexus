import { prisma } from '@/utils/prisma';
import SecurityClient, { ApiKeyType } from './SecurityClient';
import { auth } from '@/auth';

export default async function SecurityPage() {
  let keys: ApiKeyType[] = [];
  let billingPlan = 'developer';

  if (process.env.DATABASE_URL) {
    try {
      const session = await auth();
      let orgId = session?.user ? (session.user as any).organizationId : null;

      if (!orgId && session?.user?.id) {
        const userDb = await prisma.user.findUnique({
          where: { id: session.user.id },
          include: { organization: true }
        });
        if (userDb?.organization) {
          orgId = userDb.organization.id;
          billingPlan = userDb.organization.billingPlan;
        }
      } else if (orgId) {
          const org = await prisma.organization.findUnique({ where: { id: orgId } });
          if (org) billingPlan = org.billingPlan;
      }

      if (orgId) {
        const data = await prisma.apiKey.findMany({
            where: { organizationId: orgId },
            orderBy: { createdAt: 'asc' }
        });
        if (data && data.length > 0) {
            keys = data.map(k => ({
            id: k.id,
            name: k.name,
            keyHash: k.keyHash,
            createdAt: k.createdAt,
            isActive: k.isActive
            }));
        }
      }
    } catch (e) {
      console.warn("Failed to fetch API keys from DB", e);
    }
  }

  return <SecurityClient initialKeys={keys} billingPlan={billingPlan} />;
}
