import { describe, expect, it } from 'vitest';
import { decryptSecret, encryptSecret, decryptSecretAsync, encryptSecretAsync } from './VaultCrypto';

const encryptionEnv = {
  VAULT_ENCRYPTION_KEY: '1111111111111111111111111111111111111111111111111111111111111111',
};

describe('VaultCrypto', () => {
  it('round-trips encrypted secrets with the configured key', () => {
    const plainText = 'L1K-test-private-key';

    const encrypted = encryptSecret(plainText, encryptionEnv);

    expect(encrypted).not.toContain(plainText);
    expect(decryptSecret(encrypted, encryptionEnv)).toBe(plainText);
  });

  it('rejects missing encryption keys', () => {
    expect(() => encryptSecret('secret', {})).toThrow(/VAULT_ENCRYPTION_KEY/);
  });

  it('round-trips encrypted secrets using async fallback methods', async () => {
    const plainText = 'async-test-key';
    const encrypted = await encryptSecretAsync(plainText, encryptionEnv);
    expect(encrypted).not.toContain(plainText);
    const decrypted = await decryptSecretAsync(encrypted, encryptionEnv);
    expect(decrypted).toBe(plainText);
  });
});
