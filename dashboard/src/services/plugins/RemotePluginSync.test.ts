import { describe, expect, it } from 'vitest';
import {
  buildRemotePluginRemovalCommand,
  buildRemotePluginSyncCommand,
} from './RemotePluginSync';

describe('RemotePluginSync', () => {
  it('builds a safe remote sync command for a provisioned node', () => {
    const command = buildRemotePluginSyncCommand({
      host: '203.0.113.10',
      user: 'root',
      identityPath: '/home/neo/.ssh/neonexus_vm',
      remotePath: '/etc/neonexus/plugins/tee-oracle.json',
      renderedConfig: '{"pluginId":"tee-oracle"}',
    });

    expect(command).toContain('ssh');
    expect(command).toContain('/home/neo/.ssh/neonexus_vm');
    expect(command).toContain('203.0.113.10');
    expect(command).toContain('/etc/neonexus/plugins/tee-oracle.json');
    expect(command).toContain('systemctl restart neonexus-plugin-sync');
  });

  it('builds a remote removal command for a plugin config file', () => {
    const command = buildRemotePluginRemovalCommand({
      host: '203.0.113.10',
      user: 'root',
      identityPath: '/home/neo/.ssh/neonexus_vm',
      remotePath: '/etc/neonexus/plugins/tee-oracle.json',
    });

    expect(command).toContain('rm -f');
    expect(command).toContain('/etc/neonexus/plugins/tee-oracle.json');
    expect(command).toContain('systemctl restart neonexus-plugin-sync');
  });
});
