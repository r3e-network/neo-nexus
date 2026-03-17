# Operations Batch Actions Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add safe batch queue actions so operators can recover alert retries or resolve open incidents without clicking each row individually.

**Architecture:** Introduce a tiny helper that normalizes and deduplicates numeric ids for batch actions. Refactor existing single-item operations actions to share internal per-item helpers, then expose batch actions for visible alert rules and open incidents. Add one client button per queue to trigger the batch action and refresh the current page.

**Tech Stack:** Next.js server actions, TypeScript, Vitest

### Task 1: Batch Id Helper

**Files:**
- Create: `dashboard/src/app/app/operations/OperationsBatch.ts`
- Create: `dashboard/src/app/app/operations/OperationsBatch.test.ts`

**Step 1: Write the failing test**

Run: `npm run test --workspace=dashboard -- src/app/app/operations/OperationsBatch.test.ts`
Expected: FAIL because the helper does not exist.

**Step 2: Implement minimal helper**

Normalize id arrays by:
- removing duplicates
- removing non-positive / non-integer values

### Task 2: Batch Actions + Queue Buttons

**Files:**
- Modify: `dashboard/src/app/app/operations/actions.ts`
- Create: `dashboard/src/app/app/operations/RecoverAlertRulesButton.tsx`
- Create: `dashboard/src/app/app/operations/ResolveIncidentsButton.tsx`
- Modify: `dashboard/src/app/app/operations/alerts/page.tsx`
- Modify: `dashboard/src/app/app/operations/incidents/page.tsx`
- Modify: `dashboard/src/app/app/operations/OperationsAlertActions.test.ts`
- Create: `dashboard/src/app/app/operations/OperationsIncidentActions.test.ts`

**Step 1: Add batch server actions**

Support:
- recover all visible alert rules
- resolve all visible open incidents

**Step 2: Add queue buttons**

Render:
- `Recover All Visible`
- `Resolve All Visible`

### Task 3: Full Verification

Run: `npm run verify`
Expected: PASS
