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

export function parseOperatorWallets(value: string | null | undefined): Set<string> {
  return new Set(
    (value ?? '')
      .split(/[,\n]/)
      .map((wallet) => wallet.trim())
      .filter(Boolean),
  );
}

export function resolveUserRole(input: {
  role: string | null | undefined;
  walletAddress: string | null | undefined;
  operatorWallets?: Set<string>;
}): UserRole {
  const address = input.walletAddress?.trim();
  if (address && input.operatorWallets?.has(address)) {
    return 'operator';
  }

  return normalizeUserRole(input.role);
}

export function getConfiguredOperatorWallets(envValue = process.env.OPERATOR_WALLETS): Set<string> {
  return parseOperatorWallets(envValue);
}

export function assertOperatorRole(role: UserRole): asserts role is 'operator' {
  if (role !== 'operator') {
    throw new UnauthorizedError('Unauthorized: Operator access is required.');
  }
}
