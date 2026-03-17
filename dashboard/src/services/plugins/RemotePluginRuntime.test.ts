import { describe, expect, it } from 'vitest';
import {
  buildRemotePluginLogsCommand,
  buildRemotePluginStatusCommand,
  getPluginContainerName,
  parsePluginRuntimeState,
} from './RemotePluginRuntime';

describe('RemotePluginRuntime', () => {
  it('derives the expected plugin container name', () => {
    expect(getPluginContainerName('tee-oracle')).toBe('neonexus-plugin-tee-oracle');
  });

  it('parses a running docker state as active', () => {
    expect(
      parsePluginRuntimeState('{"Running":true,"Status":"running"}'),
    ).toEqual({
      status: 'Active',
      errorMessage: null,
    });
  });

  it('parses a missing container as pending apply', () => {
    expect(parsePluginRuntimeState('{}')).toEqual({
      status: 'Pending Apply',
      errorMessage: 'Plugin container is not present on the node yet.',
    });
  });

  it('parses an exited container as error', () => {
    expect(
      parsePluginRuntimeState('{"Running":false,"Status":"exited","ExitCode":1,"Error":"boom"}'),
    ).toEqual({
      status: 'Error',
      errorMessage: 'Plugin container exited with code 1. boom',
    });
  });

  it('builds remote status and logs commands for ssh execution', () => {
    const statusCommand = buildRemotePluginStatusCommand({
      host: '203.0.113.10',
      user: 'root',
      identityPath: '/home/neo/.ssh/neonexus_vm',
      pluginId: 'tee-oracle',
    });
    const logsCommand = buildRemotePluginLogsCommand({
      host: '203.0.113.10',
      user: 'root',
      identityPath: '/home/neo/.ssh/neonexus_vm',
      pluginId: 'tee-oracle',
    });

    expect(statusCommand).toContain('docker inspect');
    expect(statusCommand).toContain('neonexus-plugin-tee-oracle');
    expect(logsCommand).toContain('docker logs');
    expect(logsCommand).toContain('--tail 200');
  });
});
