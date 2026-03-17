export type MethodFirewallPresetId = 'write_methods' | 'debug_methods' | 'execution_tests';

export const METHOD_FIREWALL_PRESETS = [
  {
    id: 'write_methods',
    name: 'Block Write Methods',
    description: 'Prevent state-changing or broadcast-style RPC methods on routed public traffic.',
    methods: ['sendrawtransaction', 'submitblock'],
  },
  {
    id: 'debug_methods',
    name: 'Block Debug Methods',
    description: 'Restrict expensive state inspection methods from public traffic.',
    methods: ['getstorage', 'getcontractstate'],
  },
  {
    id: 'execution_tests',
    name: 'Block Execution Test Methods',
    description: 'Disable public invokefunction / invokescript style test execution calls.',
    methods: ['invokefunction', 'invokescript'],
  },
] as const;

export function listMethodFirewallPresets() {
  return METHOD_FIREWALL_PRESETS;
}

export function getMethodFirewallPreset(presetId: MethodFirewallPresetId) {
  return METHOD_FIREWALL_PRESETS.find((preset) => preset.id === presetId) ?? null;
}

export function isMethodFirewallPresetActive(
  presetId: MethodFirewallPresetId,
  blockedMethods: string[],
) {
  const preset = getMethodFirewallPreset(presetId);
  if (!preset) {
    return false;
  }

  const blocked = new Set(blockedMethods.map((method) => method.trim().toLowerCase()));
  return preset.methods.every((method) => blocked.has(method));
}
