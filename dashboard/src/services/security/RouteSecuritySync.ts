import { prisma } from '../../utils/prisma';
import { ApisixService } from '../apisix/ApisixService';
import { getSharedBackendTarget } from '../endpoints/SharedEndpointConfig';
import { buildRouteSecurityPlugins, type FirewallRuleType } from './FirewallPolicy';

export type OrganizationFirewallRule = {
  id?: number;
  type: FirewallRuleType;
  value: string;
};

export async function listOrganizationFirewallRules(organizationId: string): Promise<OrganizationFirewallRule[]> {
  const rules = await prisma.firewall.findMany({
    where: {
      organizationId,
      endpointId: null,
    },
    orderBy: { createdAt: 'asc' },
    select: {
      id: true,
      type: true,
      value: true,
    },
  });

  return rules.map((rule) => ({
    id: rule.id,
    type: rule.type as FirewallRuleType,
    value: rule.value,
  }));
}

export function buildOrganizationRoutePlugins(rules: OrganizationFirewallRule[]) {
  return buildRouteSecurityPlugins(rules);
}

export async function loadOrganizationRoutePlugins(organizationId: string) {
  return buildOrganizationRoutePlugins(await listOrganizationFirewallRules(organizationId));
}

export async function syncOrganizationRoutes(
  organizationId: string,
  rules: OrganizationFirewallRule[],
) {
  const routePlugins = buildOrganizationRoutePlugins(rules);
  const endpoints = await prisma.endpoint.findMany({
    where: { organizationId },
    select: {
      id: true,
      type: true,
      protocol: true,
      networkKey: true,
      url: true,
      providerPublicIp: true,
    },
  });

  const failedRouteIds: number[] = [];

  for (const endpoint of endpoints) {
    const isShared = endpoint.type.toLowerCase() === 'shared';

    if (isShared) {
      const upstream = getSharedBackendTarget(endpoint.protocol, endpoint.networkKey);
      const synced = await ApisixService.createRoute(
        String(endpoint.id),
        endpoint.url,
        upstream.host,
        upstream.port,
        routePlugins,
      );
      if (!synced) {
        failedRouteIds.push(endpoint.id);
      }
      continue;
    }

    if (!endpoint.providerPublicIp) {
      continue;
    }

    const synced = await ApisixService.createRoute(
      String(endpoint.id),
      endpoint.url,
      endpoint.providerPublicIp,
      endpoint.protocol === 'neo-x' ? 8545 : 10332,
      routePlugins,
    );
    if (!synced) {
      failedRouteIds.push(endpoint.id);
    }
  }

  if (failedRouteIds.length > 0) {
    throw new Error(`Failed to sync firewall rules to routes: ${failedRouteIds.join(', ')}`);
  }
}
