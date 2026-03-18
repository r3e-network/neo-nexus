'use server';

import { prisma } from '@/utils/prisma';
import { getErrorMessage } from '@/server/errors';
import {
  assertDatabaseConfigured,
  requireCurrentOrganizationContext,
} from '@/server/organization';
import { verifyCryptoTransferOnChain } from '@/services/billing/CryptoBillingService';

function isPlausibleTransactionHash(txHash: string): boolean {
  return /^0x[a-fA-F0-9]{64}$/.test(txHash);
}

export async function verifyCryptoPaymentAction(plan: 'growth' | 'dedicated', txHash: string) {
  try {
    assertDatabaseConfigured();
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }

  if (!isPlausibleTransactionHash(txHash)) {
    return { success: false, error: 'Transaction hash format is invalid.' };
  }

  try {
    const { organizationId } = await requireCurrentOrganizationContext();
    const normalizedTxHash = txHash.toLowerCase();

    const existingTransaction = await prisma.billingTransaction.findUnique({
      where: { txHash: normalizedTxHash },
    });

    if (existingTransaction) {
      if (existingTransaction.organizationId === organizationId && existingTransaction.plan === plan) {
        return { success: true, alreadyVerified: true };
      }

      return { success: false, error: 'This transaction hash has already been used.' };
    }

    const verification = await verifyCryptoTransferOnChain(normalizedTxHash, plan);

    await prisma.$transaction([
      prisma.billingTransaction.create({
        data: {
          organizationId,
          txHash: normalizedTxHash,
          plan,
          amountAtomic: verification.amountAtomic,
        },
      }),
      prisma.organization.update({
        where: { id: organizationId },
        data: { billingPlan: plan },
      }),
    ]);

    return { success: true };
  } catch (error) {
    console.error('Crypto verification failed:', error);
    return { success: false, error: getErrorMessage(error) };
  }
}
