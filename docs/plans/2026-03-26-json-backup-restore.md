# JSON Backup Restore Implementation Plan

> **For Implementer:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a working restore flow for the existing exported NeoNexus JSON snapshot so configuration backup/restore is complete.

**Architecture:** Reuse the current `exportConfiguration()` snapshot format and add a restore path in `NodeManager` plus a `POST /api/system/restore` endpoint. Keep restore constrained to node definitions and plugin reinstalls using the existing creation/plugin APIs, with an optional `replaceExisting` switch for destructive replacement.

**Tech Stack:** TypeScript, Express, React, TanStack Query, Vitest, Vite

### Task 1: Restore Route Coverage

**Files:**
- Modify: `tests/integration/system.router.actual.test.ts`
- Modify: `src/api/routes/system.ts`

**Step 1: Write the failing test**

Add a test that proves:
- `POST /api/system/restore` accepts a snapshot payload and returns restored/skipped/failed counts

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/integration/system.router.actual.test.ts`
Expected: FAIL because the restore route does not exist.

**Step 3: Write minimal implementation**

Add `restoreConfiguration(snapshot, options)` to the router contract and expose `/api/system/restore`.

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/integration/system.router.actual.test.ts`
Expected: PASS

### Task 2: NodeManager Restore Logic

**Files:**
- Modify: `tests/unit/NodeManager.system-actions.test.ts`
- Modify: `src/core/NodeManager.ts`

**Step 1: Write the failing test**

Cover:
- restore creates nodes from snapshot entries
- restore can optionally reset existing nodes first
- restore attempts plugin reinstalls for neo-cli nodes
- restore returns counts for restored/skipped/failed nodes

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/NodeManager.system-actions.test.ts`
Expected: FAIL because restore logic does not exist.

**Step 3: Write minimal implementation**

Implement `restoreConfiguration(snapshot, { replaceExisting })` using:
- `resetAllNodeData()` when requested
- `createNode()` with restored version/ports/settings
- `installPlugin()` for exported plugins

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/NodeManager.system-actions.test.ts`
Expected: PASS

### Task 3: Settings Restore UI

**Files:**
- Modify: `web/src/hooks/useSystemActions.ts`
- Modify: `web/src/pages/Settings.tsx`

**Step 1: Write the failing test**

Use build-safe verification for:
- restore mutation compiles
- Settings page exposes file upload and replace-existing toggle
- successful restore shows result feedback

**Step 2: Run test to verify it fails**

Run: `npm run build --prefix web`
Expected: Existing Settings page has export only, no restore action.

**Step 3: Write minimal implementation**

Add:
- restore mutation
- JSON file reader
- replace-existing checkbox
- confirmation and result/error state

**Step 4: Run test to verify it passes**

Run: `npm run build --prefix web`
Expected: PASS

### Task 4: Full Verification

**Files:**
- Modify: `README.md`

**Step 1: Run full tests**

Run: `npm test`
Expected: PASS

**Step 2: Run static verification**

Run: `npm run typecheck`
Expected: PASS

**Step 3: Run production build**

Run: `npm run build`
Expected: PASS

**Step 4: Update README roadmap**

Mark backup/restore as implemented if verification passes.
