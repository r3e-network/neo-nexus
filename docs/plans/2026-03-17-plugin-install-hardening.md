# Plugin Install Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prevent plugin installation from leaking secrets or throwing uncaught server-action errors when vault storage or config rendering fails.

**Architecture:** Keep the existing action shape, but move secret storage, config rendering, and plugin persistence into one guarded flow. Add a focused regression test that proves a stored secret is rolled back if later plugin setup fails before the plugin record becomes authoritative.

**Tech Stack:** Next.js server actions, Prisma, Vitest, TypeScript

### Task 1: Regression Test

**Files:**
- Create: `dashboard/src/app/app/endpoints/pluginActions.test.ts`
- Modify: `dashboard/src/app/app/endpoints/pluginActions.ts`

**Step 1: Write the failing test**

Run: `npm run test --workspace=dashboard -- src/app/app/endpoints/pluginActions.test.ts`
Expected: FAIL because plugin install currently leaves a stored secret behind when later setup fails.

**Step 2: Implement the minimal fix**

Guard the whole install sequence so:
- vault/config/plugin errors return `{ success: false, error }`
- newly stored secrets are best-effort deleted if install fails before success

### Task 2: Verification

**Step 1: Run focused tests**

Run:
- `npm run test --workspace=dashboard -- src/app/app/endpoints/pluginActions.test.ts`

Expected: PASS

**Step 2: Run full verification**

Run: `npm run verify`
Expected: PASS
