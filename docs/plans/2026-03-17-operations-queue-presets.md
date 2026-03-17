# Operations Queue Presets Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add operator-friendly quick presets and due-vs-scheduled timing filters so the operations queues open directly into the most relevant state.

**Architecture:** Extend the existing server-rendered queue pages with normalized query-string filters rather than adding client state. Add small quick-link preset rows per queue, introduce timing filters where the business logic already supports it, and tighten the Operations dashboard deep links so summary cards open the matching filtered queue.

**Tech Stack:** Next.js App Router, Prisma, TypeScript, Vitest

### Task 1: Queue Filter Coverage

**Files:**
- Modify: `dashboard/src/services/reconciliation/OperationsQueueFilters.ts`
- Modify: `dashboard/src/services/reconciliation/OperationsQueueFilters.test.ts`

**Step 1: Write the failing test**

Run: `npm run test --workspace=dashboard -- src/services/reconciliation/OperationsQueueFilters.test.ts`
Expected: FAIL because timing-filter normalization and preset query-string helpers do not exist yet.

**Step 2: Implement minimal helpers**

Add small helpers that:
- normalize a `timing` enum against allowed values
- serialize queue preset params into stable query strings

### Task 2: Operations Queue Presets

**Files:**
- Modify: `dashboard/src/app/app/operations/provisioning/page.tsx`
- Modify: `dashboard/src/app/app/operations/alerts/page.tsx`
- Modify: `dashboard/src/app/app/operations/incidents/page.tsx`
- Modify: `dashboard/src/app/app/operations/OperationsClient.tsx`
- Create: `dashboard/src/app/app/operations/OperationsQueuePresets.test.ts`
- Modify: `dashboard/src/app/app/operations/OperationsDeepLinks.test.ts`

**Step 1: Write the failing test**

Assert that:
- provisioning and alerts queues expose preset links
- provisioning and alerts support a `timing` filter
- the due-provisioning summary card links into the filtered queue

**Step 2: Implement minimal UI and filtering**

Add:
- quick preset chips per queue
- due/scheduled timing filters for provisioning and alerts
- filtered summary deep links from the Operations dashboard

### Task 3: Verification

**Step 1: Run focused tests**

Run:
- `npm run test --workspace=dashboard -- src/services/reconciliation/OperationsQueueFilters.test.ts`
- `npm run test --workspace=dashboard -- src/app/app/operations/OperationsQueuePresets.test.ts`
- `npm run test --workspace=dashboard -- src/app/app/operations/OperationsDeepLinks.test.ts`

Expected: PASS

**Step 2: Run full verification**

Run: `npm run verify`
Expected: PASS
