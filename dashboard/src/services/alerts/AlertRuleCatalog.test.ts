import { describe, expect, it } from 'vitest';
import {
  evaluateAlertRule,
  getAlertRuleDefinition,
  listAlertRuleDefinitions,
} from './AlertRuleCatalog';

describe('AlertRuleCatalog', () => {
  it('lists the supported alert rule definitions', () => {
    expect(listAlertRuleDefinitions().map((rule) => rule.id)).toEqual([
      'endpoint_down',
      'endpoint_stopped',
      'plugin_error',
    ]);
  });

  it('evaluates endpoint_down when the node is not active', () => {
    const rule = getAlertRuleDefinition('endpoint_down');
    expect(rule).not.toBeNull();

    const evaluation = evaluateAlertRule(rule!, {
      endpointStatus: 'Error',
      pluginStatuses: [],
    });

    expect(evaluation.triggered).toBe(true);
    expect(evaluation.message).toContain('Endpoint status is Error');
  });

  it('evaluates plugin_error when any plugin is failing', () => {
    const rule = getAlertRuleDefinition('plugin_error');
    expect(rule).not.toBeNull();

    const evaluation = evaluateAlertRule(rule!, {
      endpointStatus: 'Active',
      pluginStatuses: ['Active', 'Error'],
    });

    expect(evaluation.triggered).toBe(true);
  });
});
