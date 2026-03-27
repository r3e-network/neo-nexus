# Settings Actions Hardening Implementation Plan

> **For Implementer:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn the remaining Settings placeholder actions into working authenticated features: clean old logs, export configuration, stop all nodes, and reset node data.

**Architecture:** Add a small authenticated system router for these cross-node operations instead of stretching the node-specific router. Implement the actual filesystem/database work in `NodeManager` and `StorageManager`, then keep the web page thin by calling those endpoints and surfacing confirmation/success states.

**Tech Stack:** TypeScript, Express, better-sqlite3, React, TanStack Query, Vitest

### Task 1: System Route Coverage

**Files:**
- Create: `tests/integration/system.routes.test.ts`
- Create: `src/api/routes/system.ts`
- Modify: `src/server.ts`

**Step 1: Write the failing test**

Add route tests that prove:
- `POST /api/system/logs/clean` returns the cleaned log count
- `GET /api/system/export` returns a config snapshot payload
- `POST /api/system/nodes/stop-all` returns stopped/running counts
- `POST /api/system/reset` returns deleted node counts

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/integration/system.routes.test.ts`
Expected: FAIL because the router and endpoints do not exist.

**Step 3: Write minimal implementation**

Create `createSystemRouter(nodeManager)` and mount it behind existing auth middleware in `src/server.ts`.

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/integration/system.routes.test.ts`
Expected: PASS

### Task 2: NodeManager Operations

**Files:**
- Modify: `src/core/NodeManager.ts`
- Modify: `src/core/StorageManager.ts`

**Step 1: Write the failing test**

Cover:
- stop-all only stops running nodes
- clean-old-logs aggregates per-node counts
- export returns node configs/plugins without runtime-only noise
- reset deletes node rows and node directories

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/integration/system.routes.test.ts`
Expected: FAIL because router methods call missing manager operations.

**Step 3: Write minimal implementation**

Add methods:
- `stopAllNodes()`
- `cleanOldLogs(maxAgeDays?)`
- `exportConfiguration()`
- `resetAllNodeData()`

Keep destructive scope to node-managed data and keep users/plugin catalog intact.

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/integration/system.routes.test.ts`
Expected: PASS

### Task 3: Settings UI Wiring

**Files:**
- Modify: `web/src/pages/Settings.tsx`
- Create: `web/src/hooks/useSystemActions.ts`
- Modify: `web/src/utils/api.ts` only if needed

**Step 1: Write the failing test**

Use build-safe verification for:
- clean logs button triggers API and displays result
- export config downloads payload from API
- stop all and reset require explicit confirmation and show result/error state

**Step 2: Run test to verify it fails**

Run: `npm run build --prefix web`
Expected: Existing buttons still have no behavior.

**Step 3: Write minimal implementation**

Add mutations/hooks for system actions and wire the Settings buttons with confirmations and feedback.

**Step 4: Run test to verify it passes**

Run: `npm run build --prefix web`
Expected: PASS

### Task 4: Full Verification

**Files:**
- Modify: `README.md`

**Step 1: Run targeted and full tests**

Run: `npm test`
Expected: PASS

**Step 2: Run static verification**

Run: `npm run typecheck`
Expected: PASS

**Step 3: Run production build**

Run: `npm run build`
Expected: PASS

**Step 4: Update README status**

Mark the four Settings actions as implemented if verification passes.
