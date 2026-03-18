import { describe, expect, it } from 'vitest';
import {
  UnauthorizedError,
  assertOperatorRole,
  normalizeUserRole,
  parseOperatorWallets,
  resolveUserRole,
} from './userRoles';

describe('organization role helpers', () => {
  it('normalizes unknown roles to member and preserves operator', () => {
    expect(normalizeUserRole('operator')).toBe('operator');
    expect(normalizeUserRole('member')).toBe('member');
    expect(normalizeUserRole('unexpected')).toBe('member');
    expect(normalizeUserRole(null)).toBe('member');
  });

  it('rejects non-operator roles from operator-only access', () => {
    expect(() => assertOperatorRole('member')).toThrow(UnauthorizedError);
    expect(() => assertOperatorRole('operator')).not.toThrow();
  });

  it('treats configured operator wallets as operator access', () => {
    const operatorWallets = parseOperatorWallets('Nh7..., Nxyz...\nNabc...');

    expect(resolveUserRole({
      role: 'member',
      walletAddress: 'Nh7...',
      operatorWallets,
    })).toBe('operator');
  });

  it('falls back to the normalized stored role when wallet is not allowlisted', () => {
    const operatorWallets = parseOperatorWallets('Nh7...');

    expect(resolveUserRole({
      role: 'operator',
      walletAddress: 'NUser...',
      operatorWallets,
    })).toBe('operator');
    expect(resolveUserRole({
      role: 'unexpected',
      walletAddress: 'NUser...',
      operatorWallets,
    })).toBe('member');
  });
});
