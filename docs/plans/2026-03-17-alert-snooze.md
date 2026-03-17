# Alert Snooze Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Let operators snooze alert rule delivery for a bounded period without hiding incident state.

**Architecture:** Add `snoozedUntil` to `AlertRule`, use a pure helper to build snooze/unsnooze updates, and change alert evaluation so delivery attempts are skipped while a rule is snoozed but incidents can still remain visible. Expose the controls from the alert-rule detail page.

**Tech Stack:** Prisma, Next.js server actions, TypeScript, Vitest

### Task 1: Snooze Helper

**Files:**
- Create: `dashboard/src/services/alerts/AlertRuleSnooze.ts`
- Create: `dashboard/src/services/alerts/AlertRuleSnooze.test.ts`

**Step 1: Write the failing test**

Run: `npm run test --workspace=dashboard -- src/services/alerts/AlertRuleSnooze.test.ts`
Expected: FAIL because the helper does not exist.

**Step 2: Implement minimal helper**

Support:
- snooze until `now + duration`
- clear snooze and optionally schedule immediate reevaluation

### Task 2: Persist + Evaluate

**Files:**
- Modify: `dashboard/prisma/schema.prisma`
- Modify: `infrastructure/database/schema.sql`
- Modify: `dashboard/src/services/alerts/AlertEvaluationService.ts`
- Modify: `dashboard/src/app/app/operations/actions.ts`

### Task 3: UI

**Files:**
- Create: `dashboard/src/app/app/operations/SnoozeAlertRuleButton.tsx`
- Modify: `dashboard/src/app/app/operations/alerts/[id]/page.tsx`

### Task 4: Verification

Run: `npm run verify`
Expected: PASS
