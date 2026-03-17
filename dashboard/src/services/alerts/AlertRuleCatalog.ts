export type AlertRuleDefinition = {
  id: 'endpoint_down' | 'endpoint_stopped' | 'plugin_error';
  name: string;
  description: string;
  actionTypes: Array<'email' | 'webhook'>;
};

type EvaluationInput = {
  endpointStatus: string;
  pluginStatuses: string[];
};

const ALERT_RULES: AlertRuleDefinition[] = [
  {
    id: 'endpoint_down',
    name: 'Endpoint Down',
    description: 'Triggers when the endpoint is no longer active.',
    actionTypes: ['email', 'webhook'],
  },
  {
    id: 'endpoint_stopped',
    name: 'Endpoint Stopped',
    description: 'Triggers when the node has been stopped manually or by a provider failure.',
    actionTypes: ['email', 'webhook'],
  },
  {
    id: 'plugin_error',
    name: 'Plugin Error',
    description: 'Triggers when any installed plugin enters an error state.',
    actionTypes: ['email', 'webhook'],
  },
];

export function listAlertRuleDefinitions(): AlertRuleDefinition[] {
  return ALERT_RULES;
}

export function getAlertRuleDefinition(condition: string): AlertRuleDefinition | null {
  return ALERT_RULES.find((rule) => rule.id === condition) ?? null;
}

export function evaluateAlertRule(
  rule: AlertRuleDefinition,
  input: EvaluationInput,
) {
  switch (rule.id) {
    case 'endpoint_down':
      return {
        triggered: input.endpointStatus !== 'Active',
        severity: input.endpointStatus === 'Error' ? 'critical' : 'warning',
        message: `Endpoint status is ${input.endpointStatus}.`,
      };
    case 'endpoint_stopped':
      return {
        triggered: input.endpointStatus === 'Stopped',
        severity: 'warning',
        message: 'Endpoint has been stopped.',
      };
    case 'plugin_error':
      return {
        triggered: input.pluginStatuses.includes('Error'),
        severity: 'warning',
        message: input.pluginStatuses.includes('Error')
          ? 'One or more plugins are in an error state.'
          : 'All plugins are healthy.',
      };
  }
}
