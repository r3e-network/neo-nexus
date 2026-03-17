export function getSharedBackendTarget(protocol: string, networkKey: string) {
  const key = protocol === 'neo-x'
    ? (networkKey === 'mainnet' ? 'SHARED_NEO_X_MAINNET_UPSTREAM' : 'SHARED_NEO_X_TESTNET_UPSTREAM')
    : (networkKey === 'mainnet' ? 'SHARED_NEO_N3_MAINNET_UPSTREAM' : 'SHARED_NEO_N3_TESTNET_UPSTREAM');

  const value = process.env[key];
  if (!value) {
    throw new Error(`${key} must be configured for shared endpoint provisioning.`);
  }

  const [host, portValue] = value.split(':');
  const port = Number.parseInt(portValue, 10);

  if (!host || !Number.isInteger(port)) {
    throw new Error(`${key} must use the format host:port.`);
  }

  return { host, port };
}
