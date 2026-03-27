# Core Functionality Hardening Implementation Plan

> **For Implementer:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the highest-impact incomplete NeoNexus flows by hardening authenticated sessions, enabling working password/logout UX, wiring plugin installation/configuration from the web app, and fixing the import-node client path.

**Architecture:** Keep the existing Express + React structure, but move authentication checks to a session-aware middleware path so logout and expiry actually matter. Reuse the existing plugin and node APIs instead of inventing new endpoints, then add thin frontend hooks and forms on top of them.

**Tech Stack:** TypeScript, Express, better-sqlite3, React, TanStack Query, Vite, Vitest

### Task 1: Session-Aware Auth Coverage

**Files:**
- Modify: `tests/unit/auth.middleware.test.ts` (new)
- Modify: `tests/integration/auth.router.test.ts` (new or extend existing route coverage)
- Modify: `src/api/middleware/auth.ts`
- Modify: `src/api/routes/auth.ts`
- Modify: `src/server.ts`

**Step 1: Write the failing test**

Add tests that prove:
- a valid JWT without an active session is rejected
- `/api/auth/password` requires authentication and succeeds with an active session
- protected auth admin routes stay protected

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/auth.middleware.test.ts tests/integration/auth.router.test.ts`
Expected: FAIL because current middleware only validates JWT and current auth router mounts protected routes without route-level protection.

**Step 3: Write minimal implementation**

Implement a session-aware auth middleware factory that:
- verifies JWT
- verifies the token exists in `sessions`
- attaches the full user object to `req.user`

Apply it to protected auth routes and protected API routing.

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/auth.middleware.test.ts tests/integration/auth.router.test.ts`
Expected: PASS

### Task 2: Password And Logout UX

**Files:**
- Modify: `web/src/hooks/useAuth.tsx`
- Modify: `web/src/pages/Settings.tsx`
- Modify: `web/src/components/Layout.tsx`

**Step 1: Write the failing test**

Add coverage or assertions for:
- password change request sends the bearer token
- logout clears auth state reliably
- settings page exposes a usable password-change form

**Step 2: Run test to verify it fails**

Run: `npm run build --prefix web`
Expected: Existing UI still lacks a password-change flow and no change-password request path exists.

**Step 3: Write minimal implementation**

Add:
- `changePassword` mutation in auth hook
- settings form with current/new password
- clear loading/error/success states
- logout cleanup that clears cached user state immediately

**Step 4: Run test to verify it passes**

Run: `npm run build --prefix web`
Expected: PASS

### Task 3: Plugin Management Flow

**Files:**
- Modify: `web/src/hooks/usePlugins.ts` (new)
- Modify: `web/src/pages/Plugins.tsx`
- Modify: `web/src/pages/NodeDetail.tsx`
- Modify: `web/src/utils/api.ts`

**Step 1: Write the failing test**

Add coverage or build-safe assertions for:
- available plugins are fetched from API
- node-specific plugin install/uninstall/config mutations are callable
- plugin config updates can be submitted from the UI

**Step 2: Run test to verify it fails**

Run: `npm run build --prefix web`
Expected: Existing page is static and does not use real API hooks.

**Step 3: Write minimal implementation**

Add hooks for available and installed plugins, plus mutations for install, update, uninstall, enable, and disable. Update the plugins page to select a node and manage real plugin state.

**Step 4: Run test to verify it passes**

Run: `npm run build --prefix web`
Expected: PASS

### Task 4: Import Flow Reliability

**Files:**
- Modify: `web/src/utils/api.ts`
- Modify: `web/src/pages/ImportNode.tsx`

**Step 1: Write the failing test**

Add coverage or build-safe assertions for:
- API helpers return a consistent shape
- authenticated POST requests include bearer tokens
- import page reads the helper response shape correctly

**Step 2: Run test to verify it fails**

Run: `npm run build --prefix web`
Expected: Existing `ImportNode` expects Axios-style `response.data` while the helper returns parsed JSON directly.

**Step 3: Write minimal implementation**

Normalize the API helper and update the import page to use the returned payload directly.

**Step 4: Run test to verify it passes**

Run: `npm run build --prefix web`
Expected: PASS

### Task 5: Full Verification

**Files:**
- Modify: `README.md`

**Step 1: Run targeted tests**

Run: `npm test`
Expected: PASS

**Step 2: Run static verification**

Run: `npm run typecheck`
Expected: PASS

**Step 3: Run production builds**

Run: `npm run build`
Expected: PASS

**Step 4: Update status documentation**

Reflect the newly working flows in `README.md`.
