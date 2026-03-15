/**
 * Secret Management Service
 * 
 * Simulates integration with a secure Key Management Service (KMS) or HashiCorp Vault.
 * In a real production environment, Private Keys should NEVER be stored as plain text
 * in the database.
 */

import { prisma } from '@/utils/prisma';
import crypto from 'crypto';

export class VaultService {
    /**
     * Encrypts and securely stores a private key for a specific node plugin.
     * @param endpointId The Node ID
     * @param secretName e.g., 'OraclePrivateKey', 'BundlerKey'
     * @param plainTextKey The raw private key (e.g. WIF format)
     */
    static async storeNodeSecret(endpointId: number, secretName: string, plainTextKey: string) {
        // In production: await KMS.encrypt({ KeyId: process.env.KMS_KEY, Plaintext: plainTextKey })
        // Here we simulate encryption with a simple hash for demonstration.
        // A real system would store the ciphertext.
        const simulatedCiphertext = `kms_enc_${crypto.createHash('sha256').update(plainTextKey).digest('hex')}`;

        return await prisma.nodeSecret.upsert({
            where: {
                endpointId_name: {
                    endpointId,
                    name: secretName
                }
            },
            update: {
                secretHash: simulatedCiphertext
            },
            create: {
                endpointId,
                name: secretName,
                secretHash: simulatedCiphertext
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
}
