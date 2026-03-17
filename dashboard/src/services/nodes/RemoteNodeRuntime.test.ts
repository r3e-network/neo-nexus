import { describe, expect, it } from 'vitest';
import {
  buildRemoteNodeLogsCommand,
  buildRemoteNodeStatusCommand,
  parseNodeRuntimeState,
} from './RemoteNodeRuntime';

describe('RemoteNodeRuntime', () => {
  it('builds remote node status and log commands', () => {
    const statusCommand = buildRemoteNodeStatusCommand({
      host: '203.0.113.10',
      user: 'root',
      identityPath: '/home/neo/.ssh/neonexus_vm',
    });
    const logsCommand = buildRemoteNodeLogsCommand({
      host: '203.0.113.10',
      user: 'root',
      identityPath: '/home/neo/.ssh/neonexus_vm',
    });

    expect(statusCommand).toContain('docker inspect neonexus-node');
    expect(logsCommand).toContain('docker logs --tail 400 neonexus-node');
  });

  it('parses running node state as active', () => {
    expect(parseNodeRuntimeState('{"Running":true,"Status":"running"}')).toEqual({
      status: 'Active',
      errorMessage: null,
    });
  });

  it('parses exited node state as error', () => {
    expect(parseNodeRuntimeState('{"Running":false,"Status":"exited","ExitCode":137}')).toEqual({
      status: 'Error',
      errorMessage: 'Node container exited with code 137.',
    });
  });
});
