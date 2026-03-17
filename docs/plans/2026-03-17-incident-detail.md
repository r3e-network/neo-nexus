# Incident Detail Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an operator-facing incident detail page that shows the incident record, related alert rule state, and recent alert/activity timeline for investigation.

**Architecture:** Build a pure timeline helper that merges the incident record with recent endpoint activity rows and important alert delivery timestamps, then render a server-side operator-only page under `/app/operations/incidents/[id]`. Link the existing open-incidents drilldown page into that detail view.

**Tech Stack:** Next.js App Router, Prisma, TypeScript, Vitest

### Task 1: Incident Timeline Helper

**Files:**
- Create: `dashboard/src/services/alerts/IncidentTimeline.ts`
- Create: `dashboard/src/services/alerts/IncidentTimeline.test.ts`

**Step 1: Write the failing test**

```ts
it('builds a reverse-chronological timeline from incident and activity data', () => {
  const timeline = buildIncidentTimeline(...);
  expect(timeline[0].kind).toBe('activity');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/alerts/IncidentTimeline.test.ts`
Expected: FAIL because the helper does not exist.

**Step 3: Write minimal implementation**

Create a pure helper that returns timestamped timeline items for:
- incident opened / resolved
- delivery success/failure timestamps on the incident row
- recent endpoint activity rows related to alerts

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/alerts/IncidentTimeline.test.ts`
Expected: PASS

### Task 2: Incident Detail Page

**Files:**
- Create: `dashboard/src/app/app/operations/incidents/[id]/page.tsx`
- Modify: `dashboard/src/app/app/operations/incidents/page.tsx`

**Step 1: Implement the page**

The page should:
- require operator access
- load the incident, its alert rule, endpoint, and recent relevant endpoint activities
- render the timeline helper output

**Step 2: Link the incident list**

Add “View Details” links from the open-incidents drilldown page.

**Step 3: Verify**

Run: `npm run lint --workspace=dashboard -- src/app/app/operations/incidents/page.tsx src/app/app/operations/incidents/[id]/page.tsx`
Expected: PASS

### Task 3: Full Verification

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/services/alerts/IncidentTimeline.test.ts`
Expected: PASS

**Step 2: Run the full verification suite**

Run: `npm run verify`
Expected: PASS
