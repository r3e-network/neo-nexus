import { Resend } from 'resend';
import type { AlertRuleDefinition } from './AlertRuleCatalog';

type DeliveryInput = {
  rule: {
    id: number;
    name: string;
    actionType: string;
    target: string;
  };
  definition: AlertRuleDefinition;
  evaluation: {
    triggered: boolean;
    severity: string;
    message: string;
  };
  endpoint: {
    id: number;
    name: string;
    status: string;
  };
};

type DeliveryDependencies = {
  fetchImpl?: typeof fetch;
  env?: Record<string, string | undefined>;
};

type DeliveryStateUpdate = {
  lastTriggeredAt: Date | null;
  lastDeliveredAt: Date | null;
  lastResolvedAt: Date | null;
  lastDeliveryError: string | null;
};

type DeliveryAttemptInput = {
  alertRuleId: number;
  endpointId: number;
  actionType: string;
  target: string;
  status: 'succeeded' | 'failed';
  errorMessage?: string | null;
  now: Date;
};

export function buildAlertDeliveryStateUpdate(input: {
  triggered: boolean;
  deliverySucceeded: boolean | null;
  now: Date;
  errorMessage?: string | null;
}): DeliveryStateUpdate {
  if (!input.triggered) {
    return {
      lastTriggeredAt: null,
      lastDeliveredAt: null,
      lastResolvedAt: input.now,
      lastDeliveryError: null,
    };
  }

  return {
    lastTriggeredAt: input.now,
    lastDeliveredAt: input.deliverySucceeded ? input.now : null,
    lastResolvedAt: null,
    lastDeliveryError: input.deliverySucceeded === false ? input.errorMessage ?? 'Unknown alert delivery error.' : null,
  };
}

export function buildAlertDeliveryAttemptRecord(input: DeliveryAttemptInput) {
  return {
    alertRuleId: input.alertRuleId,
    endpointId: input.endpointId,
    actionType: input.actionType,
    target: input.target,
    status: input.status,
    errorMessage: input.errorMessage ?? null,
    createdAt: input.now,
  };
}

async function deliverWebhookNotification(
  input: DeliveryInput,
  dependencies: DeliveryDependencies,
) {
  const fetchImpl = dependencies.fetchImpl ?? fetch;
  const response = await fetchImpl(input.rule.target, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      alertRuleId: input.rule.id,
      alertRule: input.rule.name,
      severity: input.evaluation.severity,
      message: input.evaluation.message,
      endpoint: input.endpoint,
      triggered: input.evaluation.triggered,
    }),
  });

  if (!response.ok) {
    throw new Error(`Webhook delivery failed: ${response.status} ${await response.text()}`);
  }
}

async function deliverEmailNotification(
  input: DeliveryInput,
  dependencies: DeliveryDependencies,
) {
  const env = dependencies.env ?? process.env;
  const apiKey = env.RESEND_API_KEY;
  const fromEmail = env.ALERTS_FROM_EMAIL;

  if (!apiKey || !fromEmail) {
    throw new Error('RESEND_API_KEY and ALERTS_FROM_EMAIL must be configured for email alerts.');
  }

  const resend = new Resend(apiKey);
  await resend.emails.send({
    from: fromEmail,
    to: input.rule.target,
    subject: `[NeoNexus] ${input.rule.name}`,
    text: [
      `Alert: ${input.rule.name}`,
      `Endpoint: ${input.endpoint.name} (#${input.endpoint.id})`,
      `Status: ${input.endpoint.status}`,
      `Severity: ${input.evaluation.severity}`,
      `Message: ${input.evaluation.message}`,
    ].join('\n'),
  });
}

export async function deliverAlertNotification(
  input: DeliveryInput,
  dependencies: DeliveryDependencies = {},
) {
  if (input.rule.actionType === 'webhook') {
    return deliverWebhookNotification(input, dependencies);
  }

  return deliverEmailNotification(input, dependencies);
}
