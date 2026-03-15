import { auth } from '@/auth';
import { prisma } from '@/utils/prisma';

export const BILLING_PLANS = ['developer', 'growth', 'dedicated'] as const;

export type BillingPlan = (typeof BILLING_PLANS)[number];

export type UserContext = {
  userId: string;
  organizationId: string | null;
  billingPlan: BillingPlan;
};

export type OrganizationContext = UserContext & {
  organizationId: string;
};

export class UnauthorizedError extends Error {
  constructor(message = 'Unauthorized') {
    super(message);
    this.name = 'UnauthorizedError';
  }
}

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

  let organizationId = session.user.organizationId ?? null;
  let billingPlan: BillingPlan = 'developer';

  if (isDatabaseConfigured()) {
    const userRecord = await prisma.user.findUnique({
      where: { id: session.user.id },
      select: {
        organizationId: true,
        organization: {
          select: {
            billingPlan: true,
          },
        },
      },
    });

    if (userRecord?.organizationId) {
      organizationId = userRecord.organizationId;
      billingPlan = normalizeBillingPlan(userRecord.organization?.billingPlan);
    }
  }

  return {
    userId: session.user.id,
    organizationId,
    billingPlan,
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
