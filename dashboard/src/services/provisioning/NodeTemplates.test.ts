import { describe, expect, it } from 'vitest';
import { getNodeTemplates, getTemplateById } from './NodeTemplates';

describe('NodeTemplates', () => {
  it('provides official templates including RPC and Consensus', () => {
    const templates = getNodeTemplates();
    expect(templates.length).toBeGreaterThan(0);
    expect(templates.map(t => t.id)).toContain('rpc');
    expect(templates.map(t => t.id)).toContain('consensus');
  });

  it('RPC template defaults to multiple API plugins', () => {
    const rpc = getTemplateById('rpc');
    expect(rpc?.defaultPlugins).toContain('RpcServer');
    expect(rpc?.defaultPlugins).toContain('ApplicationLogs');
  });

  it('Consensus template requires DBFT', () => {
    const consensus = getTemplateById('consensus');
    expect(consensus?.defaultPlugins).toContain('DBFTPlugin');
  });
});
