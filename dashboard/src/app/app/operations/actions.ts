'use server';

import { prisma } from '@/utils/prisma';
import { recordEndpointActivity } from '@/services/endpoints/EndpointActivityService';
import { buildManualIncidentResolution } from '@/services/alerts/AlertIncidentService';
import { buildAlertRuleSnoozeUpdate } from '@/services/alerts/AlertRuleSnooze';
import { evaluateEndpointAlerts } from '@/services/alerts/AlertEvaluationService';
import { executeAndRecordReconcile } from '@/services/reconciliation/ExecuteReconcile';
import { kickoffProvisioningOrder } from '@/services/provisioning/ProvisioningRunner';
import { requireCurrentOperatorContext } from '@/server/organization';
import { normalizeBatchIds } from './OperationsBatch';

export async function runControlPlaneReconcileAction() {
  await requireCurrentOperatorContext();
  return executeAndRecordReconcile('manual');
}

async function retryProvisioningOrderById(orderId: number) {
  const order = await prisma.provisioningOrder.findUnique({
    where: {
      id: orderId,
    },
    include: {
      endpoint: {
        select: {
          id: true,
          name: true,
        },
      },
    },
  });

  if (!order) {
    return { success: false, error: 'Provisioning order not found.' };
  }

  await prisma.provisioningOrder.update({
    where: { id: order.id },
    data: {
      status: 'pending',
      currentStep: 'pending',
      nextAttemptAt: null,
      errorMessage: null,
      ...(order.status === 'failed'
        ? {
            attemptCount: 0,
            lastAttemptAt: null,
          }
        : {}),
    },
  });

  await recordEndpointActivity({
    endpointId: order.endpointId,
    category: 'provisioning',
    action: 'manual_retry',
    status: 'pending',
    message: `Manual retry requested for provisioning order #${order.id}.`,
    metadata: {
      orderId: order.id,
    },
  });

  await kickoffProvisioningOrder(order.id);
  return { success: true };
}

async function retryAlertDeliveryById(alertRuleId: number) {
  const rule = await prisma.alertRule.findUnique({
    where: {
      id: alertRuleId,
    },
    select: {
      id: true,
      name: true,
      endpointId: true,
    },
  });

  if (!rule) {
    return { success: false, error: 'Alert rule not found.' };
  }

  await prisma.alertRule.update({
    where: { id: rule.id },
    data: {
      nextDeliveryAt: new Date(0),
      lastDeliveryError: null,
    },
  });

  await recordEndpointActivity({
    endpointId: rule.endpointId,
    category: 'alert',
    action: 'manual_retry',
    status: 'pending',
    message: `Manual alert delivery retry requested for "${rule.name}".`,
    metadata: {
      alertRuleId: rule.id,
    },
  });

  await evaluateEndpointAlerts(rule.endpointId);
  return { success: true };
}

async function resolveIncidentById(incidentId: number) {
  const incident = await prisma.alertIncident.findUnique({
    where: { id: incidentId },
    select: {
      id: true,
      status: true,
      endpointId: true,
      alertRuleId: true,
      alertRule: {
        select: {
          name: true,
        },
      },
    },
  });

  if (!incident) {
    return { success: false, error: 'Incident not found.' };
  }

  if (incident.status !== 'Open') {
    return { success: false, error: 'Only open incidents can be manually resolved.' };
  }

  const now = new Date();
  const resolution = buildManualIncidentResolution(now);

  await prisma.$transaction([
    prisma.alertIncident.update({
      where: { id: incident.id },
      data: resolution,
    }),
    prisma.alertRule.update({
      where: { id: incident.alertRuleId },
      data: {
        lastResolvedAt: now,
      },
    }),
  ]);

  await recordEndpointActivity({
    endpointId: incident.endpointId,
    category: 'alert',
    action: 'incident_manually_resolved',
    status: 'success',
    message: `Alert "${incident.alertRule.name}" manually resolved by operator.`,
    metadata: {
      alertRuleId: incident.alertRuleId,
      incidentId: incident.id,
    },
  });

  return { success: true };
}

async function snoozeAlertRuleById(alertRuleId: number, snoozeMinutes: number | null) {
  const rule = await prisma.alertRule.findUnique({
    where: { id: alertRuleId },
    select: {
      id: true,
      name: true,
      lastTriggeredAt: true,
      endpointId: true,
      endpoint: {
        select: {
          alertIncidents: {
            where: {
              alertRuleId,
              status: 'Open',
            },
            take: 1,
            select: { id: true },
          },
        },
      },
    },
  });

  if (!rule) {
    return { success: false, error: 'Alert rule not found.' };
  }

  const now = new Date();
  const update = buildAlertRuleSnoozeUpdate({
    now,
    snoozeMinutes,
    hasOpenAlert: rule.endpoint.alertIncidents.length > 0 || Boolean(rule.lastTriggeredAt),
  });

  const updatedRule = await prisma.alertRule.update({
    where: { id: rule.id },
    data: update,
    select: {
      id: true,
      snoozedUntil: true,
      nextDeliveryAt: true,
    },
  });

  await recordEndpointActivity({
    endpointId: rule.endpointId,
    category: 'alert',
    action: snoozeMinutes && snoozeMinutes > 0 ? 'snoozed' : 'snooze_cleared',
    status: 'success',
    message: snoozeMinutes && snoozeMinutes > 0
      ? `Alert "${rule.name}" snoozed for ${snoozeMinutes} minutes.`
      : `Alert "${rule.name}" snooze cleared.`,
    metadata: {
      alertRuleId: rule.id,
      snoozedUntil: updatedRule.snoozedUntil?.toISOString() ?? null,
    },
  });

  return {
    success: true,
    snoozedUntil: updatedRule.snoozedUntil?.toISOString() ?? null,
    nextDeliveryAt: updatedRule.nextDeliveryAt?.toISOString() ?? null,
  };
}

async function recoverAlertRuleById(alertRuleId: number) {
  const rule = await prisma.alertRule.findUnique({
    where: { id: alertRuleId },
    select: {
      id: true,
      name: true,
      endpointId: true,
    },
  });

  if (!rule) {
    return { success: false, error: 'Alert rule not found.' };
  }

  await prisma.alertRule.update({
    where: { id: rule.id },
    data: {
      snoozedUntil: null,
      nextDeliveryAt: new Date(0),
      lastDeliveryError: null,
    },
  });

  await recordEndpointActivity({
    endpointId: rule.endpointId,
    category: 'alert',
    action: 'manual_recover',
    status: 'pending',
    message: `Alert "${rule.name}" unsnoozed and queued for immediate delivery retry.`,
    metadata: {
      alertRuleId: rule.id,
    },
  });

  await evaluateEndpointAlerts(rule.endpointId);
  return { success: true };
}

export async function retryProvisioningOrderAction(orderId: number) {
  await requireCurrentOperatorContext();
  return retryProvisioningOrderById(orderId);
}

export async function retryProvisioningOrdersAction(orderIds: number[]) {
  await requireCurrentOperatorContext();
  const ids = normalizeBatchIds(orderIds);
  let retriedCount = 0;

  for (const id of ids) {
    const result = await retryProvisioningOrderById(id);
    if (result.success) {
      retriedCount += 1;
    }
  }

  return { success: true, retriedCount };
}

export async function retryAlertDeliveryAction(alertRuleId: number) {
  await requireCurrentOperatorContext();
  return retryAlertDeliveryById(alertRuleId);
}

export async function resolveIncidentAction(incidentId: number) {
  await requireCurrentOperatorContext();
  return resolveIncidentById(incidentId);
}

export async function snoozeAlertRuleAction(alertRuleId: number, snoozeMinutes: number | null) {
  await requireCurrentOperatorContext();
  return snoozeAlertRuleById(alertRuleId, snoozeMinutes);
}

export async function recoverAlertRuleAction(alertRuleId: number) {
  await requireCurrentOperatorContext();
  return recoverAlertRuleById(alertRuleId);
}

export async function recoverAlertRulesAction(alertRuleIds: number[]) {
  await requireCurrentOperatorContext();
  const ids = normalizeBatchIds(alertRuleIds);
  let recoveredCount = 0;

  for (const id of ids) {
    const result = await recoverAlertRuleById(id);
    if (result.success) {
      recoveredCount += 1;
    }
  }

  return { success: true, recoveredCount };
}

export async function resolveIncidentsAction(incidentIds: number[]) {
  await requireCurrentOperatorContext();
  const ids = normalizeBatchIds(incidentIds);
  let resolvedCount = 0;

  for (const id of ids) {
    const result = await resolveIncidentById(id);
    if (result.success) {
      resolvedCount += 1;
    }
  }

  return { success: true, resolvedCount };
}
