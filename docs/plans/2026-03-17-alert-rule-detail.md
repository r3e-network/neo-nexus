# Alert Rule Detail Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an operator-facing alert-rule detail page with recent incidents, delivery state, and filtered alert activity history.

**Architecture:** Create a small helper that filters endpoint activity rows down to those belonging to a specific alert rule based on persisted `metadata.alertRuleId`. Use that helper in a server-rendered operator-only page under `/app/operations/alerts/[id]`, and link the alert retry queue plus incident detail page into it.

**Tech Stack:** Next.js App Router, Prisma, TypeScript, Vitest

### Task 1: Alert Activity Filter Helper

**Files:**
- Create: `dashboard/src/services/alerts/AlertRuleActivity.ts`
- Create: `dashboard/src/services/alerts/AlertRuleActivity.test.ts`

**Step 1: Write the failing test**

```ts
it('filters endpoint activities to a specific alert rule id', () => {
  const rows = filterAlertRuleActivities(...);
  expect(rows).toHaveLength(1);
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/alerts/AlertRuleActivity.test.ts`
Expected: FAIL because the helper does not exist.

**Step 3: Write minimal implementation**

Filter activity rows by `metadata.alertRuleId === ruleId`, ignoring malformed metadata.

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/alerts/AlertRuleActivity.test.ts`
Expected: PASS

### Task 2: Alert Rule Detail Page

**Files:**
- Create: `dashboard/src/app/app/operations/alerts/[id]/page.tsx`
- Modify: `dashboard/src/app/app/operations/alerts/page.tsx`
- Modify: `dashboard/src/app/app/operations/incidents/[id]/page.tsx`

**Step 1: Implement the page**

Load:
- the alert rule
- the endpoint + org
- recent incidents for that rule
- recent alert activities for that rule

**Step 2: Link into it**

Add “View Details” links from:
- alert retry queue page
- incident detail page alert-rule section

**Step 3: Verify**

Run: `npm run lint --workspace=dashboard -- src/app/app/operations/alerts/page.tsx src/app/app/operations/alerts/[id]/page.tsx src/app/app/operations/incidents/[id]/page.tsx`
Expected: PASS

### Task 3: Full Verification

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/services/alerts/AlertRuleActivity.test.ts`
Expected: PASS

**Step 2: Run the full verification suite**

Run: `npm run verify`
Expected: PASS
