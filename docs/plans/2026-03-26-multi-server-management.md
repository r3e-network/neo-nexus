# Multi-Server Management Implementation Plan

> **For Implementer:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add multi-server management by letting NeoNexus store and monitor remote NeoNexus instances through their public APIs.

**Architecture:** Persist remote server profiles in SQLite, fetch their public status/nodes/system-metrics server-side to avoid browser CORS issues, and expose authenticated CRUD/status routes to the web app. Add a dedicated Servers page that supports profile management and a remote overview without affecting local node-management flows.

**Tech Stack:** TypeScript, Express, better-sqlite3, React, TanStack Query, Vitest, Vite

### Task 1: Remote Server Manager Coverage

**Files:**
- Create: `tests/unit/RemoteServerManager.test.ts`
- Create: `src/core/RemoteServerManager.ts`
- Modify: `src/database/schema.ts`

**Step 1: Write the failing test**

Add tests that prove:
- profiles can be created/listed/updated/deleted
- base URLs are normalized
- remote public status fetch returns a reachable summary
- failed remote fetches surface a readable error state instead of crashing

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/RemoteServerManager.test.ts`
Expected: FAIL because the manager and schema table do not exist.

**Step 3: Write minimal implementation**

Create a `RemoteServerManager` with:
- CRUD methods
- `getServerSummary(id)`
- `listServersWithStatus()`

Use the remote public endpoints:
- `/api/public/status`
- `/api/public/nodes`
- `/api/public/metrics/system`

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/RemoteServerManager.test.ts`
Expected: PASS

### Task 2: Server Routes

**Files:**
- Create: `tests/integration/servers.router.actual.test.ts`
- Create: `src/api/routes/servers.ts`
- Modify: `src/server.ts`

**Step 1: Write the failing test**

Add actual route tests that prove:
- `GET /api/servers` returns profiles with live status
- `GET /api/servers/:id` returns detailed remote info
- `POST /api/servers` creates a profile
- `PUT /api/servers/:id` updates a profile
- `DELETE /api/servers/:id` removes a profile

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/integration/servers.router.actual.test.ts`
Expected: FAIL because the route file and server mount do not exist.

**Step 3: Write minimal implementation**

Create `createServersRouter(remoteServerManager)` and mount it behind auth.

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/integration/servers.router.actual.test.ts`
Expected: PASS

### Task 3: Servers Page

**Files:**
- Create: `web/src/hooks/useServers.ts`
- Create: `web/src/pages/Servers.tsx`
- Modify: `web/src/App.tsx`
- Modify: `web/src/components/Layout.tsx`

**Step 1: Write the failing test**

Use build-safe verification for:
- Servers page compiles and is routed
- server profiles can be added/edited/deleted
- remote summary cards and node lists render from the API shape

**Step 2: Run test to verify it fails**

Run: `npm run build --prefix web`
Expected: FAIL because the page, hooks, and route do not exist.

**Step 3: Write minimal implementation**

Add:
- `useServers()` and mutations
- `/servers` route and nav item
- form for profile CRUD
- remote server cards with reachability/status
- per-server node list summary

**Step 4: Run test to verify it passes**

Run: `npm run build --prefix web`
Expected: PASS

### Task 4: Full Verification

**Files:**
- Modify: `README.md`

**Step 1: Run all tests**

Run: `npm test`
Expected: PASS

**Step 2: Run static verification**

Run: `npm run typecheck`
Expected: PASS

**Step 3: Run production build**

Run: `npm run build`
Expected: PASS

**Step 4: Update README**

Mark multi-server management as implemented and update the functionality status count if needed.
