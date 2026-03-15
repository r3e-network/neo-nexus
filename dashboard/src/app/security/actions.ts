'use server';

import { prisma } from '@/utils/prisma';
import { revalidatePath } from 'next/cache';
import crypto from 'crypto';
import { ApisixService } from '@/services/apisix/ApisixService';

export async function createApiKeyAction(name: string) {
  if (!process.env.DATABASE_URL) {
    return { success: false, error: 'Database not configured' };
  }

  try {
    // Generate a secure API key
    const rawKey = 'nk_live_' + crypto.randomBytes(16).toString('hex');
    
    // In production, you'd only store the hash
    const keyHash = crypto.createHash('sha256').update(rawKey).digest('hex');

    // Default to a mock organization for demo if none exists
    let orgId = 'default-org';
    const orgs = await prisma.organization.findMany({ take: 1 });
    if (orgs.length > 0) orgId = orgs[0].id;

    await prisma.apiKey.create({
      data: {
        name: name,
        keyHash: keyHash,
        isActive: true,
        organizationId: orgId !== 'default-org' ? orgId : null
      }
    });

    // Register with APISIX API Gateway
    await ApisixService.createConsumer(orgId, rawKey, orgs[0]?.billingPlan as any || 'developer');

    revalidatePath('/security');
    
    // We return the raw key ONLY once so the user can copy it
    return { success: true, key: rawKey };
  } catch (error: any) {
    console.error('Failed to create API key:', error);
    return { success: false, error: error.message };
  }
}

export async function deleteApiKeyAction(id: string) {
  if (!process.env.DATABASE_URL) return { success: false };

  try {
    await prisma.apiKey.delete({
      where: { id }
    });
    revalidatePath('/security');
    return { success: true };
  } catch (error: any) {
    return { success: false, error: error.message };
  }
}
