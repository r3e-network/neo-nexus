import { describe, expect, it } from 'vitest';
import {
  buildRemoteResyncCommand,
  buildRemoteSnapshotCommand,
} from './RemoteMaintenance';

describe('RemoteMaintenance', () => {
  it('builds a remote resync command that clears chain data and restarts the node', () => {
    const command = buildRemoteResyncCommand({
      host: '203.0.113.10',
      user: 'root',
      identityPath: '/home/neo/.ssh/neonexus_vm',
    });

    expect(command).toContain('docker stop neonexus-node');
    expect(command).toContain('rm -rf /var/lib/neonexus');
    expect(command).toContain('docker start neonexus-node');
  });

  it('builds a remote snapshot command that archives node data', () => {
    const command = buildRemoteSnapshotCommand({
      host: '203.0.113.10',
      user: 'root',
      identityPath: '/home/neo/.ssh/neonexus_vm',
      snapshotPath: '/var/lib/neonexus/exports/node.tar.gz',
    });

    expect(command).toContain(`tar -czf '/var/lib/neonexus/exports/node.tar.gz'`);
  });
});
