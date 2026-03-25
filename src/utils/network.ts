import { createConnection } from 'node:net';
import { execAsync } from './exec';

export async function isPortAvailable(port: number, host = '0.0.0.0'): Promise<boolean> {
  return new Promise((resolve) => {
    const server = createConnection({ port, host }, () => {
      server.destroy();
      resolve(false);
    });

    server.on('error', () => {
      resolve(true);
    });

    server.setTimeout(1000, () => {
      server.destroy();
      resolve(true);
    });
  });
}

export async function findAvailablePort(startPort: number, endPort = 65535): Promise<number | null> {
  for (let port = startPort; port <= endPort && port < startPort + 100; port++) {
    if (await isPortAvailable(port)) {
      return port;
    }
  }
  return null;
}

export async function getPublicIp(): Promise<string | null> {
  try {
    const { stdout } = await execAsync('curl -s https://api.ipify.org || curl -s https://ifconfig.me');
    return stdout.trim() || null;
  } catch {
    return null;
  }
}

export async function getLocalIp(): Promise<string | null> {
  try {
    const { stdout } = await execAsync("hostname -I | awk '{print $1}'");
    return stdout.trim() || null;
  } catch {
    return null;
  }
}

export function getNetworkMagic(network: 'mainnet' | 'testnet' | 'private'): number {
  switch (network) {
    case 'mainnet':
      return 860833102;
    case 'testnet':
      return 894710606;
    case 'private':
      return 56753;
    default:
      return 860833102;
  }
}

export function getSeedList(network: 'mainnet' | 'testnet'): string[] {
  if (network === 'mainnet') {
    return [
      'seed1.neo.org:10333',
      'seed2.neo.org:10333',
      'seed3.neo.org:10333',
      'seed4.neo.org:10333',
      'seed5.neo.org:10333',
    ];
  } else {
    return [
      'seed1t.neo.org:20333',
      'seed2t.neo.org:20333',
      'seed3t.neo.org:20333',
      'seed4t.neo.org:20333',
      'seed5t.neo.org:20333',
    ];
  }
}
