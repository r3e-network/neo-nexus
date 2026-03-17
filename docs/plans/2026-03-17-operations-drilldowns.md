# Operations Drilldowns Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add operator-facing drilldown pages for pending provisioning orders, alert delivery retries, and open incidents.

**Architecture:** Keep drilldowns as server-rendered operator-only pages under `/app/operations/*` backed by direct Prisma queries. Add a small limit-clamping helper for querystring-driven page size, link the existing Operations summary cards/sections to the new pages, and avoid introducing a second client-side data path.

**Tech Stack:** Next.js App Router, Prisma, TypeScript, Vitest

### Task 1: Drilldown Limit Helper

**Files:**
- Create: `dashboard/src/services/reconciliation/OperationsDrilldownWindow.ts`
- Create: `dashboard/src/services/reconciliation/OperationsDrilldownWindow.test.ts`

**Step 1: Write the failing test**

```ts
it('clamps drilldown limits between 10 and 200', () => {
  expect(clampOperationsItemsLimit('500')).toBe(200);
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/reconciliation/OperationsDrilldownWindow.test.ts`
Expected: FAIL because the helper does not exist yet.

**Step 3: Write minimal implementation**

Implement limit parsing with a sane default and max.

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/reconciliation/OperationsDrilldownWindow.test.ts`
Expected: PASS

### Task 2: Server Drilldown Pages

**Files:**
- Create: `dashboard/src/app/app/operations/provisioning/page.tsx`
- Create: `dashboard/src/app/app/operations/alerts/page.tsx`
- Create: `dashboard/src/app/app/operations/incidents/page.tsx`
- Modify: `dashboard/src/app/app/operations/OperationsClient.tsx`

**Step 1: Implement the pages**

Each page should:
- require operator access
- support a bounded `limit` query param
- render the relevant records with endpoint links and timestamps

**Step 2: Link the existing Operations page**

Add links from:
- Pending Provisioning card/section
- Alert Delivery Retry card/section
- Open Incidents card

**Step 3: Verify**

Run: `npm run lint --workspace=dashboard -- src/app/app/operations/OperationsClient.tsx src/app/app/operations/provisioning/page.tsx src/app/app/operations/alerts/page.tsx src/app/app/operations/incidents/page.tsx`
Expected: PASS

### Task 3: Full Verification

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/services/reconciliation/OperationsDrilldownWindow.test.ts`
Expected: PASS

**Step 2: Run full verify**

Run: `npm run verify`
Expected: PASS
