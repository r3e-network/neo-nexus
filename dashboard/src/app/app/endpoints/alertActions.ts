'use server';

import { prisma } from '@/utils/prisma';
import { revalidatePath } from 'next/cache';
import { getAlertRuleDefinition } from '@/services/alerts/AlertRuleCatalog';
import { recordEndpointActivity } from '@/services/endpoints/EndpointActivityService';
import { getErrorMessage } from '@/server/errors';
import { requireCurrentOrganizationContext } from '@/server/organization';

async function requireOwnedEndpoint(endpointId: number) {
  const { organizationId } = await requireCurrentOrganizationContext();

  const endpoint = await prisma.endpoint.findFirst({
    where: {
      id: endpointId,
      organizationId,
    },
  });

  if (!endpoint) {
    throw new Error('Endpoint not found or permission denied.');
  }

  return endpoint;
}

export async function createAlertRuleAction(input: {
  endpointId: number;
  condition: string;
  actionType: 'email' | 'webhook';
  target: string;
}) {
  try {
    const endpoint = await requireOwnedEndpoint(input.endpointId);
    const definition = getAlertRuleDefinition(input.condition);
    if (!definition) {
      return { success: false, error: 'Unsupported alert rule.' };
    }

    const alertRule = await prisma.alertRule.create({
      data: {
        endpointId: endpoint.id,
        name: definition.name,
        condition: definition.id,
        actionType: input.actionType,
        target: input.target,
        isActive: true,
      },
    });

    await recordEndpointActivity({
      endpointId: endpoint.id,
      category: 'alert',
      action: 'rule_created',
      status: 'success',
      message: `Alert rule "${alertRule.name}" created.`,
      metadata: {
        alertRuleId: alertRule.id,
        condition: alertRule.condition,
        actionType: alertRule.actionType,
        target: alertRule.target,
      },
    });

    revalidatePath(`/app/endpoints/${endpoint.id}`);
    return { success: true };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}

export async function deleteAlertRuleAction(alertRuleId: number, endpointId: number) {
  try {
    const endpoint = await requireOwnedEndpoint(endpointId);

    const deletedRules = await prisma.alertRule.findMany({
      where: {
        id: alertRuleId,
        endpointId: endpoint.id,
      },
      select: {
        id: true,
        name: true,
        condition: true,
      },
    });

    await prisma.alertRule.deleteMany({
      where: {
        id: alertRuleId,
        endpointId: endpoint.id,
      },
    });

    for (const rule of deletedRules) {
      await recordEndpointActivity({
        endpointId: endpoint.id,
        category: 'alert',
        action: 'rule_deleted',
        status: 'success',
        message: `Alert rule "${rule.name}" deleted.`,
        metadata: {
          alertRuleId: rule.id,
          condition: rule.condition,
        },
      });
    }

    revalidatePath(`/app/endpoints/${endpoint.id}`);
    return { success: true };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}
