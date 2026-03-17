import { prisma } from '../../utils/prisma';
import { recordEndpointActivity } from '../endpoints/EndpointActivityService';
import { getIncidentTransition } from './AlertIncidentService';
import {
  type AlertRuleDefinition,
  evaluateAlertRule,
  getAlertRuleDefinition,
} from './AlertRuleCatalog';
import {
  buildAlertDeliveryAttemptRecord,
  deliverAlertNotification,
} from './AlertDeliveryService';
import { buildAlertDeliveryRetryUpdate } from './AlertDeliveryRetryPolicy';

export async function evaluateEndpointAlerts(endpointId: number) {
  const endpoint = await prisma.endpoint.findUnique({
    where: { id: endpointId },
    select: {
      id: true,
      name: true,
      status: true,
      alertRules: {
        orderBy: { createdAt: 'asc' },
        select: {
          id: true,
          name: true,
          condition: true,
          actionType: true,
          target: true,
          isActive: true,
          deliveryAttemptCount: true,
          nextDeliveryAt: true,
          snoozedUntil: true,
          lastTriggeredAt: true,
          lastDeliveredAt: true,
          lastResolvedAt: true,
          lastDeliveryError: true,
          createdAt: true,
        },
      },
      alertIncidents: {
        orderBy: { openedAt: 'desc' },
        take: 20,
        select: {
          id: true,
          alertRuleId: true,
          status: true,
          severity: true,
          message: true,
          openedAt: true,
          resolvedAt: true,
          lastDeliveredAt: true,
          lastDeliveryError: true,
        },
      },
      plugins: {
        select: {
          status: true,
        },
      },
    },
  });

  if (!endpoint) {
    return null;
  }

  const rules = await Promise.all(
    endpoint.alertRules.map(async (rule) => {
      const definition = getAlertRuleDefinition(rule.condition);
      const evaluation = definition
        ? evaluateAlertRule(definition, {
            endpointStatus: endpoint.status,
            pluginStatuses: endpoint.plugins.map((plugin) => plugin.status),
          })
        : {
            triggered: false,
            severity: 'warning',
            message: 'Unsupported alert rule condition.',
          };

      if (!definition) {
        return {
          ...rule,
          createdAt: rule.createdAt.toISOString(),
          triggered: evaluation.triggered,
          severity: evaluation.severity,
          message: evaluation.message,
        };
      }

      let deliveryUpdate = null;
      const openIncident = endpoint.alertIncidents.find(
        (incident) => incident.alertRuleId === rule.id && incident.status === 'Open',
      );
      const now = new Date();
      const incidentTransition = getIncidentTransition({
        triggered: evaluation.triggered,
        hasOpenIncident: Boolean(openIncident),
        severity: evaluation.severity,
        message: evaluation.message,
        now,
      });

      if (rule.isActive) {
        const deliveryDue = !rule.nextDeliveryAt || rule.nextDeliveryAt <= now;
        const isSnoozed = Boolean(rule.snoozedUntil && rule.snoozedUntil > now);
        if (evaluation.triggered && !rule.lastDeliveredAt && deliveryDue && !isSnoozed) {
          try {
            await deliverAlertNotification({
              rule: {
                id: rule.id,
                name: rule.name,
                actionType: rule.actionType,
                target: rule.target,
              },
              definition: definition as AlertRuleDefinition,
              evaluation,
              endpoint: {
                id: endpoint.id,
                name: endpoint.name,
                status: endpoint.status,
              },
            });

            deliveryUpdate = buildAlertDeliveryRetryUpdate({
              deliverySucceeded: true,
              currentAttemptCount: rule.deliveryAttemptCount,
              lastTriggeredAt: rule.lastTriggeredAt,
              now,
            });
            await prisma.alertDeliveryAttempt.create({
              data: buildAlertDeliveryAttemptRecord({
                alertRuleId: rule.id,
                endpointId: endpoint.id,
                actionType: rule.actionType,
                target: rule.target,
                status: 'succeeded',
                now,
              }),
            });
          } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown alert delivery error.';
            deliveryUpdate = buildAlertDeliveryRetryUpdate({
              deliverySucceeded: false,
              currentAttemptCount: rule.deliveryAttemptCount,
              lastTriggeredAt: rule.lastTriggeredAt,
              now,
              errorMessage,
            });
            await prisma.alertDeliveryAttempt.create({
              data: buildAlertDeliveryAttemptRecord({
                alertRuleId: rule.id,
                endpointId: endpoint.id,
                actionType: rule.actionType,
                target: rule.target,
                status: 'failed',
                errorMessage,
                now,
              }),
            });
          }
        } else if (!evaluation.triggered && rule.lastTriggeredAt) {
          deliveryUpdate = buildAlertDeliveryRetryUpdate({
            deliverySucceeded: null,
            currentAttemptCount: rule.deliveryAttemptCount,
            lastTriggeredAt: rule.lastTriggeredAt,
            now,
          });
        }
      }

      if (incidentTransition.action === 'open') {
        await prisma.alertIncident.create({
          data: {
            alertRuleId: rule.id,
            endpointId: endpoint.id,
            status: 'Open',
            severity: incidentTransition.severity,
            message: incidentTransition.message,
            openedAt: incidentTransition.openedAt,
            lastDeliveredAt: deliveryUpdate?.lastDeliveredAt ?? null,
            lastDeliveryError: deliveryUpdate?.lastDeliveryError ?? null,
          },
        });
        await recordEndpointActivity({
          endpointId: endpoint.id,
          category: 'alert',
          action: 'incident_opened',
          status: 'error',
          message: `Alert "${rule.name}" triggered: ${incidentTransition.message}`,
          metadata: {
            alertRuleId: rule.id,
            severity: incidentTransition.severity,
          },
        });
      } else if (incidentTransition.action === 'resolve' && openIncident) {
        await prisma.alertIncident.update({
          where: { id: openIncident.id },
          data: {
            status: 'Resolved',
            resolvedAt: incidentTransition.resolvedAt,
          },
        });
        await recordEndpointActivity({
          endpointId: endpoint.id,
          category: 'alert',
          action: 'incident_resolved',
          status: 'success',
          message: `Alert "${rule.name}" resolved.`,
          metadata: {
            alertRuleId: rule.id,
          },
        });
      }

      if (deliveryUpdate) {
        await prisma.alertRule.update({
          where: { id: rule.id },
          data: {
            ...deliveryUpdate,
            ...(!evaluation.triggered ? { snoozedUntil: null } : {}),
          },
        });

        if (deliveryUpdate.lastDeliveredAt) {
          await recordEndpointActivity({
            endpointId: endpoint.id,
            category: 'alert',
            action: 'delivery_succeeded',
            status: 'success',
            message: `Alert "${rule.name}" delivered to ${rule.target}.`,
            metadata: {
              alertRuleId: rule.id,
              actionType: rule.actionType,
              target: rule.target,
            },
          });
        } else if (deliveryUpdate.lastDeliveryError) {
          await recordEndpointActivity({
            endpointId: endpoint.id,
            category: 'alert',
            action: 'delivery_failed',
            status: 'error',
            message: `Alert "${rule.name}" delivery failed.`,
            metadata: {
              alertRuleId: rule.id,
              actionType: rule.actionType,
              target: rule.target,
              error: deliveryUpdate.lastDeliveryError,
            },
          });
        }
      }

      return {
        ...rule,
        ...deliveryUpdate,
        createdAt: rule.createdAt.toISOString(),
        lastTriggeredAt: (deliveryUpdate?.lastTriggeredAt ?? rule.lastTriggeredAt)?.toISOString?.() ?? null,
        lastDeliveredAt: (deliveryUpdate?.lastDeliveredAt ?? rule.lastDeliveredAt)?.toISOString?.() ?? null,
        lastResolvedAt: (deliveryUpdate?.lastResolvedAt ?? rule.lastResolvedAt)?.toISOString?.() ?? null,
        lastDeliveryError: deliveryUpdate?.lastDeliveryError ?? rule.lastDeliveryError ?? null,
        triggered: evaluation.triggered,
        severity: evaluation.severity,
        message: evaluation.message,
      };
    }),
  );

  const incidents = await prisma.alertIncident.findMany({
    where: { endpointId: endpoint.id },
    orderBy: { openedAt: 'desc' },
    take: 20,
  });

  return {
    rules,
    incidents: incidents.map((incident) => ({
      ...incident,
      openedAt: incident.openedAt.toISOString(),
      resolvedAt: incident.resolvedAt?.toISOString() ?? null,
      lastDeliveredAt: incident.lastDeliveredAt?.toISOString() ?? null,
    })),
  };
}
