'use server';

import { StripeService } from '@/services/billing/StripeService';
import { redirect } from 'next/navigation';
import { prisma } from '@/utils/prisma';
import { getErrorMessage } from '@/server/errors';
import {
  assertDatabaseConfigured,
  requireCurrentOrganizationContext,
} from '@/server/organization';
import { verifyCryptoTransferOnChain } from '@/services/billing/CryptoBillingService';

function getAppBaseUrl(): string {
  if (process.env.NEXT_PUBLIC_APP_URL) {
    return process.env.NEXT_PUBLIC_APP_URL;
  }

  if (process.env.VERCEL_URL) {
    return `https://${process.env.VERCEL_URL}`;
  }

  return 'http://localhost:3000';
}

function isPlausibleTransactionHash(txHash: string): boolean {
  return /^0x[a-fA-F0-9]{64}$/.test(txHash);
}

export async function upgradePlanAction(plan: 'growth' | 'dedicated') {
  const { organizationId } = await requireCurrentOrganizationContext();
  const baseUrl = getAppBaseUrl();

  const { url } = await StripeService.createCheckoutSession(
    organizationId,
    plan,
    `${baseUrl}/app/billing?success=true`,
    `${baseUrl}/app/billing?canceled=true`,
  );

  redirect(url);
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

    console.log(`[Crypto Billing] Verifying TX ${normalizedTxHash} for plan ${plan}...`);

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

    console.log(`[Crypto Billing] Organization ${organizationId} upgraded to ${plan} via Web3 payment.`);
    return { success: true };
  } catch (error) {
    console.error('Crypto verification failed:', error);
    return { success: false, error: getErrorMessage(error) };
  }
}
