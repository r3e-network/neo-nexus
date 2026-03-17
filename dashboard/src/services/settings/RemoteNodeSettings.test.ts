import { describe, expect, it } from 'vitest';
import { buildRemoteNodeSettingsSyncCommand } from './RemoteNodeSettings';

describe('RemoteNodeSettings', () => {
  it('builds a remote command to write settings artifacts and run node sync', () => {
    const command = buildRemoteNodeSettingsSyncCommand({
      host: '203.0.113.10',
      user: 'root',
      identityPath: '/home/neo/.ssh/neonexus_vm',
      settingsPath: '/etc/neonexus/node-settings.json',
      renderedSettings: '{"maxPeers": 100}',
      syncScriptPath: '/usr/local/bin/neonexus-node-sync',
      renderedSyncScript: '#!/usr/bin/env bash\necho sync',
    });

    expect(command).toContain('node-settings.json');
    expect(command).toContain('neonexus-node-sync');
  });
});
