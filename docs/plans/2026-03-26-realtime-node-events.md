# Realtime Node Events Implementation Plan

> **For Implementer:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add live node status and log broadcasting over WebSocket and harden the web client connection behavior so realtime features are reliable instead of metrics-only.

**Architecture:** Promote `NodeManager` into an event source for node lifecycle and log events, then have the existing server WebSocket layer subscribe and broadcast structured messages. Keep the frontend change small by adding reconnect/backoff behavior to the socket hook and consuming live log/status events in the node detail view.

**Tech Stack:** TypeScript, Express, ws, React, TanStack Query, Vitest, Vite

### Task 1: NodeManager Event Coverage

**Files:**
- Create: `tests/unit/NodeManager.events.test.ts`
- Modify: `src/core/NodeManager.ts`

**Step 1: Write the failing test**

Add tests that prove:
- `startNode` emits a status event when node status changes
- node log events are re-emitted by `NodeManager`
- stop/restart paths emit consistent status payloads

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/NodeManager.events.test.ts`
Expected: FAIL because `NodeManager` is not currently an event emitter.

**Step 3: Write minimal implementation**

Make `NodeManager` extend `EventEmitter`, define event types, and emit:
- `nodeStatus`
- `nodeLog`
- `nodeMetrics`

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/NodeManager.events.test.ts`
Expected: PASS

### Task 2: Server WebSocket Broadcast Coverage

**Files:**
- Create: `tests/unit/realtimeMessages.test.ts`
- Modify: `src/server.ts`

**Step 1: Write the failing test**

Cover the message-shape helper or broadcaster contract for:
- `status` messages
- `log` messages
- existing `metrics` and `system` message compatibility

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/realtimeMessages.test.ts`
Expected: FAIL because no shared message builders exist for realtime event broadcasting.

**Step 3: Write minimal implementation**

Extract small message-builder helpers and subscribe the server to `NodeManager` events to broadcast status/log/metrics messages immediately.

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/realtimeMessages.test.ts`
Expected: PASS

### Task 3: Web Client Realtime Consumption

**Files:**
- Modify: `web/src/hooks/useWebSocket.tsx`
- Modify: `web/src/pages/NodeDetail.tsx`

**Step 1: Write the failing test**

Use build-safe verification for:
- reconnect/backoff support in the WebSocket hook
- live log append behavior in node detail
- reduced noisy socket errors on expected reconnects

**Step 2: Run test to verify it fails**

Run: `npm run build --prefix web`
Expected: Existing hook is one-shot, no reconnect, and node detail only polls logs.

**Step 3: Write minimal implementation**

Add:
- reconnect timer with cleanup
- parsed message typing
- live log append for current node
- optional query invalidation for realtime status/metrics

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

**Step 4: Update README**

Mark the realtime items that are now verified as working.
