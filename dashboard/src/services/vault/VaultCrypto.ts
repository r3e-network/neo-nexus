import crypto from 'crypto';
import { KMSClient, EncryptCommand, DecryptCommand } from '@aws-sdk/client-kms';

const ENCRYPTION_PREFIX = 'enc:v1';
const AWS_KMS_PREFIX = 'kms:v1';
const IV_LENGTH = 12;

let kmsClient: KMSClient | null = null;

function getKmsClient(env: Record<string, string | undefined> = process.env): KMSClient {
  if (!kmsClient) {
    kmsClient = new KMSClient({
      region: env.AWS_REGION || 'us-east-1',
    });
  }
  return kmsClient;
}

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

// Legacy Sync Methods (Local Encryption)
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

// New Async Methods (Supports KMS & Fallback to Local)
export async function encryptSecretAsync(plainText: string, env: Record<string, string | undefined> = process.env): Promise<string> {
  if (env.KMS_KEY_ID) {
    const client = getKmsClient(env);
    const command = new EncryptCommand({
      KeyId: env.KMS_KEY_ID,
      Plaintext: Buffer.from(plainText, 'utf8'),
    });
    const response = await client.send(command);
    if (!response.CiphertextBlob) {
      throw new Error('KMS encryption failed to return CiphertextBlob.');
    }
    return `${AWS_KMS_PREFIX}:${Buffer.from(response.CiphertextBlob).toString('base64url')}`;
  }
  return encryptSecret(plainText, env);
}

export async function decryptSecretAsync(ciphertext: string, env: Record<string, string | undefined> = process.env): Promise<string> {
  if (ciphertext.startsWith(AWS_KMS_PREFIX)) {
    const [, encryptedValue] = ciphertext.split(':');
    const client = getKmsClient(env);
    const command = new DecryptCommand({
      CiphertextBlob: Buffer.from(encryptedValue, 'base64url'),
    });
    const response = await client.send(command);
    if (!response.Plaintext) {
      throw new Error('KMS decryption failed to return Plaintext.');
    }
    return Buffer.from(response.Plaintext).toString('utf8');
  }
  return decryptSecret(ciphertext, env);
}
