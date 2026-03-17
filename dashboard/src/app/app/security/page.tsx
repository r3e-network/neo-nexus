import { prisma } from '@/utils/prisma';
import SecurityClient, { ApiKeyType } from './SecurityClient';
import { getCurrentUserContext } from '@/server/organization';

export const dynamic = 'force-dynamic';

export default async function SecurityPage() {
  let keys: ApiKeyType[] = [];
  let firewallRules: Array<{
    id: number;
    type: 'ip_allow' | 'origin_allow' | 'method_block';
    value: string;
    createdAt: string;
  }> = [];
  let billingPlan = 'developer';

  if (process.env.DATABASE_URL) {
    try {
      const userContext = await getCurrentUserContext();
      const orgId = userContext?.organizationId ?? null;

      if (userContext) {
        billingPlan = userContext.billingPlan;
      }

      if (orgId) {
        const [apiKeys, firewalls] = await Promise.all([
          prisma.apiKey.findMany({
            where: { organizationId: orgId },
            orderBy: { createdAt: 'asc' },
          }),
          prisma.firewall.findMany({
            where: {
              organizationId: orgId,
              endpointId: null,
            },
            orderBy: { createdAt: 'asc' },
          }),
        ]);

        if (apiKeys.length > 0) {
          keys = apiKeys.map((key) => ({
            id: key.id,
            name: key.name,
            keyHash: key.keyHash,
            createdAt: key.createdAt,
            isActive: key.isActive,
          }));
        }

        firewallRules = firewalls.map((rule) => ({
          id: rule.id,
          type: rule.type as 'ip_allow' | 'origin_allow' | 'method_block',
          value: rule.value,
          createdAt: rule.createdAt.toISOString(),
        }));
      }
    } catch (error) {
      console.warn('Failed to fetch API keys from DB', error);
    }
  }

  return <SecurityClient initialKeys={keys} initialFirewallRules={firewallRules} billingPlan={billingPlan} />;
}
