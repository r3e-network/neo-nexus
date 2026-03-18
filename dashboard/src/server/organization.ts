import { auth } from '../auth';
import { prisma } from '../utils/prisma';
import {
  UnauthorizedError,
  assertOperatorRole,
  getConfiguredOperatorWallets,
  resolveUserRole,
} from './userRoles';

export const BILLING_PLANS = ['developer', 'growth', 'dedicated'] as const;

export type BillingPlan = (typeof BILLING_PLANS)[number];
export type { UserRole } from './userRoles';
import type { UserRole } from './userRoles';

export type UserContext = {
  userId: string;
  name: string | null;
  organizationId: string | null;
  billingPlan: BillingPlan;
  role: UserRole;
  walletAddress: string | null;
};

export type OrganizationContext = UserContext & {
  organizationId: string;
};

export type OperatorContext = UserContext & {
  role: 'operator';
};

export class MissingOrganizationError extends Error {
  constructor(message = 'No organization found for this user. Please complete onboarding.') {
    super(message);
    this.name = 'MissingOrganizationError';
  }
}

export class DatabaseConfigurationError extends Error {
  constructor(message = 'Database is not configured for this environment.') {
    super(message);
    this.name = 'DatabaseConfigurationError';
  }
}

export function isDatabaseConfigured(): boolean {
  return Boolean(process.env.DATABASE_URL);
}

export function assertDatabaseConfigured(): void {
  if (!isDatabaseConfigured()) {
    throw new DatabaseConfigurationError();
  }
}

export function normalizeBillingPlan(plan: string | null | undefined): BillingPlan {
  if (plan && BILLING_PLANS.includes(plan as BillingPlan)) {
    return plan as BillingPlan;
  }

  return 'developer';
}

export async function getCurrentUserContext(): Promise<UserContext | null> {
  const session = await auth();

  if (!session?.user?.id) {
    return null;
  }

  const operatorWallets = getConfiguredOperatorWallets();
  let organizationId = session.user.organizationId ?? null;
  let billingPlan: BillingPlan = 'developer';
  let role: UserRole = resolveUserRole({
    role: session.user.role,
    walletAddress: session.user.walletAddress,
    operatorWallets,
  });

  if (isDatabaseConfigured()) {
    const userRecord = await prisma.user.findUnique({
      where: { id: session.user.id },
      select: {
        walletAddress: true,
        role: true,
        organizationId: true,
        organization: {
          select: {
            billingPlan: true,
          },
        },
      },
    });

    if (userRecord) {
      organizationId = userRecord.organizationId ?? null;
      billingPlan = normalizeBillingPlan(userRecord.organization?.billingPlan);
      role = resolveUserRole({
        role: userRecord.role,
        walletAddress: userRecord.walletAddress ?? session.user.walletAddress,
        operatorWallets,
      });
    }
  }

  return {
    userId: session.user.id,
    name: session.user.name || null,
    walletAddress: session.user.walletAddress || null,
    organizationId,
    billingPlan,
    role,
  };
}

export async function requireCurrentUserContext(): Promise<UserContext> {
  const context = await getCurrentUserContext();

  if (!context) {
    throw new UnauthorizedError('Unauthorized: You must be logged in.');
  }

  return context;
}

export async function requireCurrentOrganizationContext(): Promise<OrganizationContext> {
  const context = await requireCurrentUserContext();

  if (!context.organizationId) {
    throw new MissingOrganizationError();
  }

  return context as OrganizationContext;
}

export async function requireCurrentOperatorContext(): Promise<OperatorContext> {
  const context = await requireCurrentUserContext();
  assertOperatorRole(context.role);
  return context as OperatorContext;
}
