export const USER_ROLES = ['member', 'operator'] as const;

export type UserRole = (typeof USER_ROLES)[number];

export class UnauthorizedError extends Error {
  constructor(message = 'Unauthorized') {
    super(message);
    this.name = 'UnauthorizedError';
  }
}

export function normalizeUserRole(role: string | null | undefined): UserRole {
  if (role && USER_ROLES.includes(role as UserRole)) {
    return role as UserRole;
  }

  return 'member';
}

export function parseOperatorEmails(value: string | null | undefined): Set<string> {
  return new Set(
    (value ?? '')
      .split(/[,\n]/)
      .map((email) => email.trim().toLowerCase())
      .filter(Boolean),
  );
}

export function resolveUserRole(input: {
  role: string | null | undefined;
  email: string | null | undefined;
  operatorEmails?: Set<string>;
}): UserRole {
  const normalizedEmail = input.email?.trim().toLowerCase();
  if (normalizedEmail && input.operatorEmails?.has(normalizedEmail)) {
    return 'operator';
  }

  return normalizeUserRole(input.role);
}

export function getConfiguredOperatorEmails(envValue = process.env.OPERATOR_EMAILS): Set<string> {
  return parseOperatorEmails(envValue);
}

export function assertOperatorRole(role: UserRole): asserts role is 'operator' {
  if (role !== 'operator') {
    throw new UnauthorizedError('Unauthorized: Operator access is required.');
  }
}
