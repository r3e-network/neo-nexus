/**
 * Secret Management Service
 * 
 * Encrypts secrets before they are persisted so raw private keys are never stored
 * directly in the database.
 */

import { prisma } from '@/utils/prisma';
import { decryptSecret, encryptSecret } from './VaultCrypto';

export class VaultService {
    /**
     * Encrypts and securely stores a private key for a specific node plugin.
     * @param endpointId The Node ID
     * @param secretName e.g., 'OraclePrivateKey', 'BundlerKey'
     * @param plainTextKey The raw private key (e.g. WIF format)
     */
    static async storeNodeSecret(endpointId: number, secretName: string, plainTextKey: string) {
        const ciphertext = encryptSecret(plainTextKey);

        return await prisma.nodeSecret.upsert({
            where: {
                endpointId_name: {
                    endpointId,
                    name: secretName
                }
            },
            update: {
                secretHash: ciphertext
            },
            create: {
                endpointId,
                name: secretName,
                secretHash: ciphertext
            }
        });
    }

    /**
     * Checks if a specific secret exists for a node, without revealing its value.
     */
    static async hasSecret(endpointId: number, secretName: string): Promise<boolean> {
        const secret = await prisma.nodeSecret.findUnique({
            where: {
                endpointId_name: {
                    endpointId,
                    name: secretName
                }
            }
        });
        return !!secret;
    }

    static async readNodeSecret(endpointId: number, secretName: string): Promise<string | null> {
        const secret = await prisma.nodeSecret.findUnique({
            where: {
                endpointId_name: {
                    endpointId,
                    name: secretName
                }
            }
        });

        if (!secret) {
            return null;
        }

        return decryptSecret(secret.secretHash);
    }
}
