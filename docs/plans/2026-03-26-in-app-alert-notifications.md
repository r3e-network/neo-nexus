# In-App Alert Notifications Implementation Plan

> **For Implementer:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add working in-app alert notifications for important realtime node events such as status transitions and error logs.

**Architecture:** Reuse the existing WebSocket message flow and derive notifications on the client from `status` and `log` messages. Keep the feature self-contained with a notifications provider, a small reducer/helper for event-to-notification mapping, and a compact bell/toast UI in the existing layout.

**Tech Stack:** TypeScript, React, React Router, TanStack Query, ws, Vitest, Vite

### Task 1: Notification Mapping Coverage

**Files:**
- Create: `tests/unit/notifications.test.ts`
- Create: `web/src/utils/notifications.ts`

**Step 1: Write the failing test**

Add tests that prove:
- error-level node logs create alert notifications
- status transitions like `error`, `stopped`, and `running` create readable notifications
- duplicate events can be deduplicated by key

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/notifications.test.ts`
Expected: FAIL because the notification helpers do not exist.

**Step 3: Write minimal implementation**

Create mapping helpers for:
- `notificationFromRealtimeMessage`
- `dedupeNotifications`

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/notifications.test.ts`
Expected: PASS

### Task 2: Notifications Provider

**Files:**
- Create: `web/src/hooks/useNotifications.tsx`
- Modify: `web/src/App.tsx`

**Step 1: Write the failing test**

Use build-safe verification for:
- provider compiles and exposes unread notifications
- provider consumes the existing WebSocket context

**Step 2: Run test to verify it fails**

Run: `npm run build --prefix web`
Expected: FAIL because the provider and hook do not exist.

**Step 3: Write minimal implementation**

Add a notifications provider that:
- listens to `lastMessage`
- appends deduplicated notifications
- marks notifications read
- dismisses notifications

**Step 4: Run test to verify it passes**

Run: `npm run build --prefix web`
Expected: PASS

### Task 3: UI Surface In Layout

**Files:**
- Modify: `web/src/components/Layout.tsx`
- Modify: `web/src/index.css` only if needed

**Step 1: Write the failing test**

Use build-safe verification for:
- bell badge renders unread count
- dropdown lists recent alerts
- toast stack shows latest notifications

**Step 2: Run test to verify it fails**

Run: `npm run build --prefix web`
Expected: Existing layout has no notification UI.

**Step 3: Write minimal implementation**

Add:
- header bell button
- recent notifications dropdown
- small dismissible toast stack

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

Reflect that alert notifications are now implemented in-app.
