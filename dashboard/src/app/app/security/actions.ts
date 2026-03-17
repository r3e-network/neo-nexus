'use server';

import { prisma } from '@/utils/prisma';
import { revalidatePath } from 'next/cache';
import crypto from 'crypto';
import { ApisixService } from '@/services/apisix/ApisixService';
import { getErrorMessage } from '@/server/errors';
import {
  normalizeFirewallRuleValue,
  type FirewallRuleType,
} from '@/services/security/FirewallPolicy';
import {
  getMethodFirewallPreset,
  type MethodFirewallPresetId,
} from '@/services/security/MethodFirewallPresets';
import {
  listOrganizationFirewallRules,
  syncOrganizationRoutes,
} from '@/services/security/RouteSecuritySync';
import {
  assertDatabaseConfigured,
  requireCurrentOrganizationContext,
} from '@/server/organization';

export async function createApiKeyAction(name: string) {
  let createdApiKeyId: string | null = null;

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

    const apiKey = await prisma.apiKey.create({
      data: {
        name: name,
        keyHash: keyHash,
        isActive: true,
        organizationId: orgId
      }
    });
    createdApiKeyId = apiKey.id;

    // Register with APISIX API Gateway
    const synced = await ApisixService.createConsumer(apiKey.id, rawKey, billingPlan);
    if (!synced) {
      await prisma.apiKey.delete({
        where: { id: apiKey.id },
      });
      createdApiKeyId = null;
      return { success: false, error: 'Failed to sync the new API key to APISIX.' };
    }

    revalidatePath('/app/security');
    
    // We return the raw key ONLY once so the user can copy it
    return {
      success: true,
      key: rawKey,
      apiKey: {
        id: apiKey.id,
        name: apiKey.name,
        keyHash: apiKey.keyHash,
        createdAt: apiKey.createdAt.toISOString(),
        isActive: apiKey.isActive,
      },
    };
  } catch (error) {
    if (createdApiKeyId) {
      try {
        await prisma.apiKey.delete({
          where: { id: createdApiKeyId },
        });
      } catch {
        // Best-effort rollback for partially-created keys.
      }
    }
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

    const apiKey = await prisma.apiKey.findFirst({
      where: {
        id,
        organizationId,
      },
      select: {
        id: true,
      },
    });

    if (!apiKey) {
      return { success: false, error: 'API key not found or permission denied.' };
    }

    const synced = await ApisixService.deleteConsumer(apiKey.id);
    if (!synced) {
      return { success: false, error: 'Failed to remove the API key from APISIX.' };
    }

    await prisma.apiKey.delete({
      where: { id: apiKey.id },
    });

    revalidatePath('/app/security');
    return { success: true };
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }
}

export async function createFirewallRuleAction(type: FirewallRuleType, value: string) {
  let organizationId: string | null = null;
  let existingRules: Array<{ id?: number; type: FirewallRuleType; value: string }> = [];
  let syncedNextRules = false;

  try {
    assertDatabaseConfigured();
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }

  try {
    ({ organizationId } = await requireCurrentOrganizationContext());
    const normalizedValue = normalizeFirewallRuleValue(type, value);
    existingRules = await listOrganizationFirewallRules(organizationId);

    if (existingRules.some((rule) => rule.type === type && rule.value === normalizedValue)) {
      return { success: false, error: 'This firewall rule already exists.' };
    }

    const nextRules = [...existingRules, { type, value: normalizedValue }];
    await syncOrganizationRoutes(organizationId, nextRules);
    syncedNextRules = true;

    const firewall = await prisma.firewall.create({
      data: {
        organizationId,
        endpointId: null,
        type,
        value: normalizedValue,
      },
    });

    revalidatePath('/app/security');
    revalidatePath('/app/endpoints');
    return {
      success: true,
      rule: {
        id: firewall.id,
        type,
        value: firewall.value,
        createdAt: firewall.createdAt.toISOString(),
      },
    };
  } catch (error) {
    if (syncedNextRules && organizationId) {
      try {
        await syncOrganizationRoutes(organizationId, existingRules);
      } catch {
        // Preserve the original error while attempting a best-effort rollback.
      }
    }
    return { success: false, error: getErrorMessage(error) };
  }
}

export async function deleteFirewallRuleAction(id: number) {
  let organizationId: string | null = null;
  let existingRules: Array<{ id?: number; type: FirewallRuleType; value: string }> = [];
  let syncedNextRules = false;

  try {
    assertDatabaseConfigured();
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }

  try {
    ({ organizationId } = await requireCurrentOrganizationContext());
    existingRules = await listOrganizationFirewallRules(organizationId);
    const firewall = existingRules.find((rule) => rule.id === id);

    if (!firewall) {
      return { success: false, error: 'Firewall rule not found or permission denied.' };
    }

    const nextRules = existingRules.filter((rule) => rule.id !== id);
    await syncOrganizationRoutes(organizationId, nextRules);
    syncedNextRules = true;

    await prisma.firewall.delete({
      where: { id },
    });

    revalidatePath('/app/security');
    revalidatePath('/app/endpoints');
    const updatedRules = await prisma.firewall.findMany({
      where: {
        organizationId,
        endpointId: null,
      },
      orderBy: { createdAt: 'asc' },
      select: {
        id: true,
        type: true,
        value: true,
        createdAt: true,
      },
    });

    return {
      success: true,
      rules: updatedRules.map((rule) => ({
        id: rule.id,
        type: rule.type,
        value: rule.value,
        createdAt: rule.createdAt.toISOString(),
      })),
    };
  } catch (error) {
    if (syncedNextRules && organizationId) {
      try {
        await syncOrganizationRoutes(organizationId, existingRules);
      } catch {
        // Preserve the original error while attempting a best-effort rollback.
      }
    }
    return { success: false, error: getErrorMessage(error) };
  }
}

export async function setMethodFirewallPresetAction(
  presetId: MethodFirewallPresetId,
  enabled: boolean,
) {
  let organizationId: string | null = null;
  let existingRules: Array<{ id?: number; type: FirewallRuleType; value: string }> = [];
  let syncedNextRules = false;

  try {
    assertDatabaseConfigured();
  } catch (error) {
    return { success: false, error: getErrorMessage(error) };
  }

  try {
    const context = await requireCurrentOrganizationContext();
    organizationId = context.organizationId;
    const orgId = context.organizationId;
    const { billingPlan } = context;
    if (billingPlan === 'developer') {
      return { success: false, error: 'Method firewall presets require the Growth or Dedicated plan.' };
    }

    const preset = getMethodFirewallPreset(presetId);
    if (!preset) {
      return { success: false, error: 'Unknown method firewall preset.' };
    }

    existingRules = await listOrganizationFirewallRules(orgId);
    const methodRules = existingRules.filter((rule) => rule.type === 'method_block');
    const otherRules = existingRules.filter((rule) => rule.type !== 'method_block');
    const currentMethodValues = new Set(methodRules.map((rule) => rule.value));
    const presetValues = preset.methods.map((method) => normalizeFirewallRuleValue('method_block', method));

    const nextMethodValues = enabled
      ? new Set([...currentMethodValues, ...presetValues])
      : new Set([...currentMethodValues].filter((value) => !presetValues.includes(value)));
    const nextRules = [
      ...otherRules,
      ...Array.from(nextMethodValues).map((value) => ({
        type: 'method_block' as const,
        value,
      })),
    ];

    await syncOrganizationRoutes(orgId, nextRules);
    syncedNextRules = true;

    await prisma.$transaction(async (tx) => {
      if (enabled) {
        const existingMethodSet = new Set(methodRules.map((rule) => rule.value));
        for (const value of presetValues) {
          if (existingMethodSet.has(value)) {
            continue;
          }
          await tx.firewall.create({
            data: {
              organizationId: orgId,
              endpointId: null,
              type: 'method_block',
              value,
            },
          });
        }
        return;
      }

      await tx.firewall.deleteMany({
        where: {
          organizationId: orgId,
          endpointId: null,
          type: 'method_block',
          value: {
            in: presetValues,
          },
        },
      });
    });

    revalidatePath('/app/security');
    revalidatePath('/app/endpoints');
    return { success: true };
  } catch (error) {
    if (syncedNextRules && organizationId) {
      try {
        await syncOrganizationRoutes(organizationId, existingRules);
      } catch {
        // Preserve the original error while attempting a best-effort rollback.
      }
    }
    return { success: false, error: getErrorMessage(error) };
  }
}
