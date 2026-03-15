import crypto from 'crypto';

const ENCRYPTION_PREFIX = 'enc:v1';
const IV_LENGTH = 12;

function parseEncryptionKey(rawKey: string): Buffer {
  const trimmedKey = rawKey.trim();

  if (/^[0-9a-fA-F]{64}$/.test(trimmedKey)) {
    return Buffer.from(trimmedKey, 'hex');
  }

  const buffer = Buffer.from(trimmedKey, 'base64');
  if (buffer.length === 32) {
    return buffer;
  }

  throw new Error('VAULT_ENCRYPTION_KEY must be a 32-byte hex or base64 value.');
}

export function getVaultEncryptionKey(env: Record<string, string | undefined> = process.env): Buffer {
  const rawKey = env.VAULT_ENCRYPTION_KEY;

  if (!rawKey) {
    throw new Error('VAULT_ENCRYPTION_KEY must be configured for secret storage.');
  }

  return parseEncryptionKey(rawKey);
}

export function encryptSecret(plainText: string, env: Record<string, string | undefined> = process.env): string {
  const key = getVaultEncryptionKey(env);
  const iv = crypto.randomBytes(IV_LENGTH);
  const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);
  const encrypted = Buffer.concat([cipher.update(plainText, 'utf8'), cipher.final()]);
  const authTag = cipher.getAuthTag();

  return [
    ENCRYPTION_PREFIX,
    iv.toString('base64url'),
    authTag.toString('base64url'),
    encrypted.toString('base64url'),
  ].join(':');
}

export function decryptSecret(ciphertext: string, env: Record<string, string | undefined> = process.env): string {
  const [prefix, version, ivValue, authTagValue, encryptedValue] = ciphertext.split(':');

  if (`${prefix}:${version}` !== ENCRYPTION_PREFIX || !ivValue || !authTagValue || !encryptedValue) {
    throw new Error('Stored secret is not a valid encrypted payload.');
  }

  const key = getVaultEncryptionKey(env);
  const iv = Buffer.from(ivValue, 'base64url');
  const authTag = Buffer.from(authTagValue, 'base64url');
  const encrypted = Buffer.from(encryptedValue, 'base64url');
  const decipher = crypto.createDecipheriv('aes-256-gcm', key, iv);
  decipher.setAuthTag(authTag);

  return Buffer.concat([decipher.update(encrypted), decipher.final()]).toString('utf8');
}
