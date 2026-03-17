import { describe, expect, it } from 'vitest';
import {
  buildRemoteNodeMetricsCommand,
  parseNodeMetricsSnapshot,
} from './RemoteNodeMetrics';

describe('RemoteNodeMetrics', () => {
  it('builds the expected combined remote metrics command', () => {
    const command = buildRemoteNodeMetricsCommand({
      host: '203.0.113.10',
      user: 'root',
      identityPath: '/home/neo/.ssh/neonexus_vm',
    });

    expect(command).toContain('docker stats neonexus-node --no-stream');
    expect(command).toContain('df -B1 /var/lib/neonexus');
  });

  it('parses docker and disk metrics into numeric values', () => {
    const snapshot = parseNodeMetricsSnapshot(
      [
        '{"CPUPerc":"12.5%","MemUsage":"1.5GiB / 8GiB","NetIO":"10MB / 20MB","BlockIO":"30MB / 40MB"}',
        '1610612736,8589934592',
      ].join('\n'),
    );

    expect(snapshot.cpuPercent).toBe(12.5);
    expect(snapshot.memoryUsedBytes).toBe(1610612736);
    expect(snapshot.memoryLimitBytes).toBe(8589934592);
    expect(snapshot.networkRxBytes).toBe(10_000_000);
    expect(snapshot.networkTxBytes).toBe(20_000_000);
    expect(snapshot.blockReadBytes).toBe(30_000_000);
    expect(snapshot.blockWriteBytes).toBe(40_000_000);
    expect(snapshot.diskUsedBytes).toBe(1610612736);
    expect(snapshot.diskTotalBytes).toBe(8589934592);
  });
});
