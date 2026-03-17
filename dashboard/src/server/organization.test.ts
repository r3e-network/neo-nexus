import { describe, expect, it } from 'vitest';
import {
  UnauthorizedError,
  assertOperatorRole,
  normalizeUserRole,
  parseOperatorEmails,
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

  it('treats configured operator emails as operator access', () => {
    const operatorEmails = parseOperatorEmails('ops@example.com, admin@example.com\nowner@example.com');

    expect(resolveUserRole({
      role: 'member',
      email: 'Admin@Example.com',
      operatorEmails,
    })).toBe('operator');
  });

  it('falls back to the normalized stored role when email is not allowlisted', () => {
    const operatorEmails = parseOperatorEmails('ops@example.com');

    expect(resolveUserRole({
      role: 'operator',
      email: 'user@example.com',
      operatorEmails,
    })).toBe('operator');
    expect(resolveUserRole({
      role: 'unexpected',
      email: 'user@example.com',
      operatorEmails,
    })).toBe('member');
  });
});
