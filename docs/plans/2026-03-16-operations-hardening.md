# Operations Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prevent customer access to platform-global control-plane operations data and make the operations summary reflect real retry and scheduler state.

**Architecture:** Introduce an explicit user role in auth and organization context, then gate operations routes, actions, and navigation behind an operator check. Tighten the summary service so due retries include past-due work and scheduler health is driven by the latest scheduled heartbeat instead of only the last success timestamp.

**Tech Stack:** Next.js App Router, NextAuth, Prisma, Vitest, TypeScript

### Task 1: Role-Aware User Context

**Files:**
- Modify: `dashboard/prisma/schema.prisma`
- Modify: `infrastructure/database/schema.sql`
- Modify: `dashboard/src/auth.ts`
- Modify: `dashboard/src/types/next-auth.d.ts`
- Modify: `dashboard/src/server/organization.ts`
- Test: `dashboard/src/server/organization.test.ts`

**Step 1: Write the failing test**

```ts
it('normalizes unknown roles to member and recognizes operator users', async () => {
  expect(normalizeUserRole('operator')).toBe('operator');
  expect(normalizeUserRole('unexpected')).toBe('member');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/server/organization.test.ts`
Expected: FAIL because role helpers and operator context do not exist yet.

**Step 3: Write minimal implementation**

```ts
export const USER_ROLES = ['member', 'operator'] as const;
export type UserRole = (typeof USER_ROLES)[number];
```

Add role propagation in auth/session callbacks and enforce `requireCurrentOperatorContext`.

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/server/organization.test.ts`
Expected: PASS

### Task 2: Operations Authorization Surface

**Files:**
- Modify: `dashboard/src/app/api/operations/reconcile/route.ts`
- Modify: `dashboard/src/app/api/operations/reconcile/[id]/route.ts`
- Modify: `dashboard/src/app/api/operations/summary/route.ts`
- Modify: `dashboard/src/app/app/operations/actions.ts`
- Modify: `dashboard/src/app/app/operations/page.tsx`
- Modify: `dashboard/src/app/app/operations/[id]/page.tsx`
- Modify: `dashboard/src/app/app/layout.tsx`
- Modify: `dashboard/src/components/Sidebar.tsx`
- Modify: `dashboard/src/app/app/page.tsx`

**Step 1: Write the failing test**

```ts
it('throws when a non-operator requests operator context', async () => {
  await expect(requireCurrentOperatorContext()).rejects.toThrow(UnauthorizedError);
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/server/organization.test.ts`
Expected: FAIL because operator-only guard is not implemented.

**Step 3: Write minimal implementation**

Require operator context in operations routes/actions/pages, pass `showOperations` to the sidebar, and only render overview control-plane widgets for operators.

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/server/organization.test.ts`
Expected: PASS

### Task 3: Scheduler And Retry Summary Logic

**Files:**
- Modify: `dashboard/src/services/reconciliation/OperationsSummaryService.ts`
- Modify: `dashboard/src/services/reconciliation/OperationsSummaryService.test.ts`
- Modify: `dashboard/src/app/api/operations/summary/route.ts`
- Modify: `dashboard/src/app/app/operations/OperationsClient.tsx`

**Step 1: Write the failing test**

```ts
it('counts past-due retries as due and marks recent scheduled failures as degraded', () => {
  const summary = buildOperationsSummary({
    pendingOrders: [{ nextAttemptAt: '2026-03-16T11:50:00.000Z' }],
    latestScheduledRunAt: '2026-03-16T11:59:00.000Z',
    latestScheduledRunStatus: 'error',
    now: new Date('2026-03-16T12:00:00.000Z'),
  });

  expect(summary.dueProvisioningCount).toBe(1);
  expect(summary.schedulerStatus).toBe('degraded');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/reconciliation/OperationsSummaryService.test.ts`
Expected: FAIL because due retries only count null timestamps and scheduler status does not model degraded runs.

**Step 3: Write minimal implementation**

Compute due retries for null or `<= now`, track the latest scheduled heartbeat timestamp/status, and expose `schedulerStatus` plus `latestScheduledRunAt`.

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/reconciliation/OperationsSummaryService.test.ts`
Expected: PASS

### Task 4: Full Verification

**Files:**
- Modify: `README.md` if role/operator behavior needs documentation

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/server/organization.test.ts src/services/reconciliation/OperationsSummaryService.test.ts`
Expected: PASS

**Step 2: Run the full verification suite**

Run: `npm run verify`
Expected: PASS

**Step 3: Review the diff**

Run: `git diff -- dashboard/prisma/schema.prisma dashboard/src/auth.ts dashboard/src/server/organization.ts dashboard/src/app/api/operations dashboard/src/app/app/operations dashboard/src/app/app/page.tsx dashboard/src/components/Sidebar.tsx dashboard/src/services/reconciliation/OperationsSummaryService.ts`
Expected: Only operator gating and summary logic changes.
