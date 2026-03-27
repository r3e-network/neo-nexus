# Production Hardening & Visibility Implementation Plan

> **For Implementer:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Improve NeoNexus production readiness by making secure-signer state visible in the main operator views and by hardening node-route validation/error handling so misconfiguration returns clear user-facing responses instead of generic 500s.

**Architecture:** Keep the current Express + React layout, but move more validation into the route boundary with consistent status mapping and surface secure-signer protection/health as first-class node metadata in the dashboard and nodes list through small child components that use the existing signer-health endpoint.

**Tech Stack:** TypeScript, Express, React, TanStack Query, Vitest

### Task 1: Node Route Validation Hardening

**Files:**
- Modify: `src/api/routes/nodes.ts`
- Test: `tests/integration/nodes.router.actual.test.ts` (new)

**Step 1: Write the failing test**

Add route tests that prove:
- creating a node with secure-signer protection but no signer profile returns `400`
- creating a `neo-go` node with secure-signer protection returns `400`
- validation/not-found errors return `400/404` instead of `500`

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/integration/nodes.router.actual.test.ts`
Expected: FAIL because the create route currently forwards these errors as `500`.

**Step 3: Write minimal implementation**

Add a small node-route error mapper and secure-signer request validation on create/update.

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/integration/nodes.router.actual.test.ts`
Expected: PASS

### Task 2: Main Operator Visibility

**Files:**
- Modify: `web/src/pages/Nodes.tsx`
- Modify: `web/src/pages/Dashboard.tsx`
- Modify: `web/src/hooks/useNodes.ts`

**Step 1: Write the failing test**

Prefer build-safe assertions over shallow UI tests. Add or extend tests only where needed for helper logic.

**Step 2: Run test/build to verify it fails**

Run: `npm run build --prefix web`
Expected: FAIL if new secure-signer state is referenced without matching types/hooks.

**Step 3: Write minimal implementation**

Add:
- secure-signer protection badges in the nodes list
- node-card visibility for protected nodes on the dashboard
- signer readiness/status pill in main operator views using the signer-health endpoint

**Step 4: Run test/build to verify it passes**

Run: `npm run build --prefix web`
Expected: PASS

### Task 3: Final Verification

**Files:**
- Modify: `README.md` if needed

**Step 1: Run targeted tests**

Run: `npm test -- tests/integration/nodes.router.actual.test.ts`
Expected: PASS

**Step 2: Run full suite**

Run: `npm test`
Expected: PASS

**Step 3: Run static verification**

Run: `npm run typecheck`
Expected: PASS

**Step 4: Run production build**

Run: `npm run build`
Expected: PASS
