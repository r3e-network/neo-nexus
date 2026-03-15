'use server';

import { prisma } from '@/utils/prisma';
import { revalidatePath } from 'next/cache';
import crypto from 'crypto';
import { ApisixService } from '@/services/apisix/ApisixService';
import { getErrorMessage } from '@/server/errors';
import {
  assertDatabaseConfigured,
  requireCurrentOrganizationContext,
} from '@/server/organization';

export async function createApiKeyAction(name: string) {
  try {
    assertDatabaseConfigured();
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }

  try {
    const { organizationId: orgId, billingPlan } = await requireCurrentOrganizationContext();

    // Check API Key limits based on plan
    const keyCount = await prisma.apiKey.count({ where: { organizationId: orgId } });
    if (billingPlan === 'developer' && keyCount >= 2) {
      return { success: false, error: 'Developer plan is limited to 2 API keys. Please upgrade to Growth.' };
    }
    if (billingPlan === 'growth' && keyCount >= 10) {
      return { success: false, error: 'Growth plan is limited to 10 API keys.' };
    }

    // Generate a secure API key
    const rawKey = 'nk_live_' + crypto.randomBytes(16).toString('hex');
    
    // In production, you'd only store the hash
    const keyHash = crypto.createHash('sha256').update(rawKey).digest('hex');

    await prisma.apiKey.create({
      data: {
        name: name,
        keyHash: keyHash,
        isActive: true,
        organizationId: orgId
      }
    });

    // Register with APISIX API Gateway
    await ApisixService.createConsumer(orgId, rawKey, billingPlan);

    revalidatePath('/app/security');
    
    // We return the raw key ONLY once so the user can copy it
    return { success: true, key: rawKey };
  } catch (error) {
    console.error('Failed to create API key:', error);
    return { success: false, error: getErrorMessage(error) };
  }
}

export async function deleteApiKeyAction(id: string) {
  try {
    assertDatabaseConfigured();
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }

  try {
    const { organizationId } = await requireCurrentOrganizationContext();

    const deleted = await prisma.apiKey.deleteMany({
      where: {
        id,
        organizationId,
      },
    });

    if (!deleted.count) {
      return { success: false, error: 'API key not found or permission denied.' };
    }

    revalidatePath('/app/security');
    return { success: true };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}
