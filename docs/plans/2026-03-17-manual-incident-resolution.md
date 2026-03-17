# Manual Incident Resolution Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Let operators manually resolve open incidents from the incident detail page, with clear persisted audit history.

**Architecture:** Add a small pure helper for the manual incident resolution update payload, then use a new operator-only server action to update the incident row, touch the related alert rule timestamps, and record endpoint activity. Expose that action through a client-side button on the incident detail page that refreshes the current view after success.

**Tech Stack:** Next.js server actions, Prisma, TypeScript, Vitest

### Task 1: Resolution Helper

**Files:**
- Modify: `dashboard/src/services/alerts/AlertIncidentService.ts`
- Create: `dashboard/src/services/alerts/AlertIncidentService.test.ts`

**Step 1: Write the failing test**

```ts
it('builds a manual resolution update for an open incident', () => {
  const update = buildManualIncidentResolution(new Date('2026-03-17T00:00:00Z'));
  expect(update.status).toBe('Resolved');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/alerts/AlertIncidentService.test.ts`
Expected: FAIL because the helper does not exist yet.

**Step 3: Write minimal implementation**

Return the incident update payload used by the server action.

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/alerts/AlertIncidentService.test.ts`
Expected: PASS

### Task 2: Operator Action + UI

**Files:**
- Modify: `dashboard/src/app/app/operations/actions.ts`
- Create: `dashboard/src/app/app/operations/ResolveIncidentButton.tsx`
- Modify: `dashboard/src/app/app/operations/incidents/[id]/page.tsx`

**Step 1: Add the action**

Implement an operator-only server action that:
- verifies the incident exists and is open
- marks it resolved
- updates the related alert rule `lastResolvedAt`
- records endpoint activity

**Step 2: Add the button**

Render a resolve button on the incident detail page only when the incident is open.

**Step 3: Verify**

Run: `npm run lint --workspace=dashboard -- src/app/app/operations/actions.ts src/app/app/operations/ResolveIncidentButton.tsx src/app/app/operations/incidents/[id]/page.tsx`
Expected: PASS

### Task 3: Full Verification

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/services/alerts/AlertIncidentService.test.ts`
Expected: PASS

**Step 2: Run full verify**

Run: `npm run verify`
Expected: PASS
