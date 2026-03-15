'use server';

import { prisma } from '@/utils/prisma';
import { revalidatePath } from 'next/cache';
import crypto from 'crypto';
import { ApisixService } from '@/services/apisix/ApisixService';
import { auth } from '@/auth';

export async function createApiKeyAction(name: string) {
  if (!process.env.DATABASE_URL) {
    return { success: false, error: 'Database not configured' };
  }

  try {
    const session = await auth();
    if (!session?.user) {
      return { success: false, error: 'Unauthorized: You must be logged in.' };
    }

    let orgId = (session.user as any).organizationId;
    let billingPlan = 'developer';

    if (!orgId) {
      const userDb = await prisma.user.findUnique({
        where: { id: session.user.id },
        include: { organization: true }
      });
      if (userDb?.organization) {
        orgId = userDb.organization.id;
        billingPlan = userDb.organization.billingPlan;
      } else {
        return { success: false, error: 'User does not belong to an organization.' };
      }
    } else {
      const org = await prisma.organization.findUnique({ where: { id: orgId } });
      if (org) billingPlan = org.billingPlan;
    }

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
    await ApisixService.createConsumer(orgId, rawKey, billingPlan as any);

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
