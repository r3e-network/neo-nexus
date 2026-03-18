import { fetchRemoteNodeMetrics } from '../dashboard/src/services/nodes/RemoteNodeMetrics';
import { fetchRemoteNodeRuntimeStatus, fetchRemoteNodeLogs } from '../dashboard/src/services/nodes/RemoteNodeRuntime';

async function main() {
  const sshInput = {
    host: '91.99.197.255',
    user: 'root',
    identityPath: '/home/neo/.ssh/id_ed25519'
  };

  console.log('--- Fetching Node Status ---');
  try {
    const status = await fetchRemoteNodeRuntimeStatus(sshInput);
    console.log('Status Result:', status);
  } catch (err) {
    console.error('Failed to get status:', err);
  }

  console.log('\n--- Fetching Node Metrics ---');
  try {
    const metrics = await fetchRemoteNodeMetrics(sshInput);
    console.log('Metrics Result:', metrics);
  } catch (err) {
    console.error('Failed to get metrics:', err);
  }

  console.log('\n--- Fetching Node Logs (Tail 5) ---');
  try {
    const logs = await fetchRemoteNodeLogs(sshInput);
    console.log('Logs (first 5 lines):\\n' + logs.split('\\n').slice(0, 5).join('\\n'));
  } catch (err) {
    console.error('Failed to get logs:', err);
  }
}

main().catch(console.error);
