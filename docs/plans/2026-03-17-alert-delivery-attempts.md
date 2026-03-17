# Alert Delivery Attempts Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Persist explicit alert delivery attempt records and surface them in the alert-rule and incident detail pages.

**Architecture:** Add an `AlertDeliveryAttempt` model tied to `AlertRule` and `Endpoint`. Create a small helper that builds the persisted attempt payload from the current delivery context, write one row for every actual delivery attempt in `evaluateEndpointAlerts`, then query and render those attempts in the existing operator drilldown pages.

**Tech Stack:** Prisma, Next.js App Router, TypeScript, Vitest

### Task 1: Delivery Attempt Payload Helper

**Files:**
- Modify: `dashboard/src/services/alerts/AlertDeliveryService.ts`
- Modify: `dashboard/src/services/alerts/AlertDeliveryService.test.ts`

**Step 1: Write the failing test**

```ts
it('builds a persisted delivery attempt payload', () => {
  const attempt = buildAlertDeliveryAttemptRecord(...);
  expect(attempt.status).toBe('failed');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/alerts/AlertDeliveryService.test.ts`
Expected: FAIL because the helper does not exist yet.

**Step 3: Write minimal implementation**

Add a helper that returns a serializable row payload for delivery attempts with:
- rule id
- endpoint id
- action type / target snapshot
- status
- error message
- created timestamp

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/alerts/AlertDeliveryService.test.ts`
Expected: PASS

### Task 2: Persist Attempts

**Files:**
- Modify: `dashboard/prisma/schema.prisma`
- Modify: `infrastructure/database/schema.sql`
- Modify: `dashboard/src/services/alerts/AlertEvaluationService.ts`

**Step 1: Add the model**

Create `AlertDeliveryAttempt` linked to `AlertRule` and `Endpoint`.

**Step 2: Persist attempts**

Write a row whenever an actual email/webhook delivery attempt is made, whether it succeeds or fails.

**Step 3: Verify**

Run: `npm run typecheck --workspace=dashboard`
Expected: PASS

### Task 3: Surface Attempts In Operator Pages

**Files:**
- Modify: `dashboard/src/app/app/operations/alerts/[id]/page.tsx`
- Modify: `dashboard/src/app/app/operations/incidents/[id]/page.tsx`

**Step 1: Query recent attempts**

Load recent delivery attempts for the relevant rule and render them.

**Step 2: Verify**

Run: `npm run lint --workspace=dashboard -- src/app/app/operations/alerts/[id]/page.tsx src/app/app/operations/incidents/[id]/page.tsx`
Expected: PASS

### Task 4: Full Verification

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/services/alerts/AlertDeliveryService.test.ts`
Expected: PASS

**Step 2: Run full verify**

Run: `npm run verify`
Expected: PASS
