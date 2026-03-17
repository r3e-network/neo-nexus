import { describe, expect, it } from 'vitest';
import {
  buildRouteSecurityPlugins,
  normalizeFirewallRuleValue,
} from './FirewallPolicy';
import {
  isMethodFirewallPresetActive,
  listMethodFirewallPresets,
} from './MethodFirewallPresets';

describe('FirewallPolicy', () => {
  it('normalizes valid ip allowlist entries', () => {
    expect(normalizeFirewallRuleValue('ip_allow', ' 203.0.113.10 ')).toBe('203.0.113.10');
    expect(normalizeFirewallRuleValue('ip_allow', '203.0.113.0/24')).toBe('203.0.113.0/24');
  });

  it('normalizes valid origin allowlist entries', () => {
    expect(normalizeFirewallRuleValue('origin_allow', 'https://app.example.com/')).toBe('https://app.example.com');
  });

  it('rejects malformed firewall values', () => {
    expect(() => normalizeFirewallRuleValue('ip_allow', 'example.com')).toThrow();
    expect(() => normalizeFirewallRuleValue('origin_allow', 'not-a-url')).toThrow();
    expect(() => normalizeFirewallRuleValue('method_block', 'bad method')).toThrow();
  });

  it('builds APISIX plugins from ip and origin allowlists', () => {
    const plugins = buildRouteSecurityPlugins([
      { type: 'ip_allow', value: '203.0.113.10' },
      { type: 'ip_allow', value: '203.0.113.0/24' },
      { type: 'origin_allow', value: 'https://app.example.com' },
    ]);

    expect(plugins['ip-restriction']).toMatchObject({
      whitelist: ['203.0.113.10', '203.0.113.0/24'],
      message: 'Source IP is not allowed.',
    });
    expect(plugins.cors).toMatchObject({
      allow_origins: 'https://app.example.com',
      allow_methods: 'GET,POST,OPTIONS',
    });
  });

  it('builds a serverless-pre-function plugin for blocked json-rpc methods', () => {
    const plugins = buildRouteSecurityPlugins([
      { type: 'method_block', value: 'getstorage' },
      { type: 'method_block', value: 'sendrawtransaction' },
    ]);

    expect(plugins['serverless-pre-function']).toMatchObject({
      phase: 'access',
    });
    const functions = (plugins['serverless-pre-function'] as { functions: string[] }).functions;
    expect(functions[0]).toContain('RPC method blocked by organization policy');
    expect(functions[0]).toContain('["getstorage"]=true');
    expect(functions[0]).toContain('["sendrawtransaction"]=true');
  });

  it('derives preset activation from stored method block rules', () => {
    const presets = listMethodFirewallPresets();
    expect(presets.map((preset) => preset.id)).toEqual([
      'write_methods',
      'debug_methods',
      'execution_tests',
    ]);
    expect(isMethodFirewallPresetActive('write_methods', ['sendrawtransaction', 'submitblock'])).toBe(true);
    expect(isMethodFirewallPresetActive('debug_methods', ['getstorage'])).toBe(false);
  });
});
