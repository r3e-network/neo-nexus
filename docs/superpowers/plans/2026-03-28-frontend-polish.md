# Frontend Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the NeoNexus frontend complete, correct, beautiful, consistent with backend business logic, and secure.

**Architecture:** Add missing pages (Setup, User Management, Audit Log), standardize loading/empty states across all pages, add form validation, improve visual consistency, and fix UX gaps.

**Tech Stack:** React 19, React Router 7, TanStack React Query 5, Tailwind CSS 4, Lucide React icons

---

## File Structure

### New Files
- `web/src/pages/Setup.tsx` — First-time setup page (create initial admin account)
- `web/src/pages/settings/UserManagement.tsx` — Admin user management (list/create/delete users)
- `web/src/pages/settings/AuditLogSection.tsx` — Audit log viewer
- `web/src/components/LoadingSkeleton.tsx` — Reusable skeleton loader components
- `web/src/components/EmptyState.tsx` — Reusable empty state with icon, title, description, action
- `web/src/components/SpinnerButton.tsx` — Button with loading spinner state
- `web/src/hooks/useUsers.ts` — Hooks for user management API
- `web/src/hooks/useAuditLog.ts` — Hook for audit log API

### Modified Files
- `web/src/App.tsx` — Add setup route, 404 catch-all, setup redirect
- `web/src/hooks/useAuth.tsx` — Add setup status check
- `web/src/pages/Login.tsx` — Add setup redirect
- `web/src/pages/Settings.tsx` — Add UserManagement and AuditLog sections
- `web/src/pages/Dashboard.tsx` — Use LoadingSkeleton
- `web/src/pages/Nodes.tsx` — Use LoadingSkeleton, SpinnerButton
- `web/src/pages/Servers.tsx` — Use LoadingSkeleton, EmptyState
- `web/src/pages/Plugins.tsx` — Use LoadingSkeleton
- `web/src/pages/NodeDetail.tsx` — Add network height, use SpinnerButton
- `web/src/pages/CreateNode.tsx` — Add numeric validation
- `web/src/components/FeedbackBanner.tsx` — Add entrance animation
- `web/src/pages/node-detail/NodeLogsView.tsx` — Use EmptyState

---

## Task 1: Setup Page & First-Time Experience

Create the initial setup page and integrate it into the auth flow so first-time users can create their admin account.

**Files:**
- Create: `web/src/pages/Setup.tsx`
- Modify: `web/src/App.tsx`
- Modify: `web/src/pages/Login.tsx`

- [ ] **Step 1: Create Setup.tsx** — A page with username/password form that calls `POST /api/auth/setup`. On success, stores token and redirects to dashboard. Should have the same dark theme as Login.
- [ ] **Step 2: Update App.tsx** — Add `/setup` route. Add 404 catch-all route that redirects to `/`.
- [ ] **Step 3: Update Login.tsx** — On mount, check `GET /api/auth/setup-status`. If `needsSetup` is true, redirect to `/setup`.
- [ ] **Step 4: Verify** — `cd web && npx tsc --noEmit`
- [ ] **Step 5: Commit** — `feat(web): add initial setup page for first-time admin creation`

---

## Task 2: Reusable Loading & Empty State Components

Create shared components to standardize loading and empty states across all pages.

**Files:**
- Create: `web/src/components/LoadingSkeleton.tsx`
- Create: `web/src/components/EmptyState.tsx`
- Create: `web/src/components/SpinnerButton.tsx`

- [ ] **Step 1: Create LoadingSkeleton.tsx** — Export `CardSkeleton`, `TableRowSkeleton`, `StatSkeleton` components using shimmer animation with Tailwind.
- [ ] **Step 2: Create EmptyState.tsx** — Accepts `icon`, `title`, `description`, optional `action` (label + onClick or href). Centered layout with muted icon.
- [ ] **Step 3: Create SpinnerButton.tsx** — Wraps a button with automatic spinner when `loading` prop is true. Disables button during loading. Preserves children text.
- [ ] **Step 4: Verify** — `cd web && npx tsc --noEmit`
- [ ] **Step 5: Commit** — `feat(web): add LoadingSkeleton, EmptyState, and SpinnerButton components`

---

## Task 3: Apply Loading/Empty States Across Pages

Replace all "Loading..." text and plain empty states with the new components.

**Files:**
- Modify: `web/src/pages/Dashboard.tsx`
- Modify: `web/src/pages/Nodes.tsx`
- Modify: `web/src/pages/Servers.tsx`
- Modify: `web/src/pages/Plugins.tsx`
- Modify: `web/src/pages/node-detail/NodeLogsView.tsx`

- [ ] **Step 1: Update Dashboard.tsx** — Use `StatSkeleton` while metrics load. Use `EmptyState` for no-nodes state.
- [ ] **Step 2: Update Nodes.tsx** — Use `TableRowSkeleton` while loading. Use `EmptyState` for no-nodes. Use `SpinnerButton` for start/stop actions.
- [ ] **Step 3: Update Servers.tsx** — Replace "Loading remote servers..." with `CardSkeleton`. Replace "No remote servers" with `EmptyState`.
- [ ] **Step 4: Update Plugins.tsx** — Replace "Loading features..." with `CardSkeleton`.
- [ ] **Step 5: Update NodeLogsView.tsx** — Replace "No logs available" with `EmptyState`.
- [ ] **Step 6: Verify** — `cd web && npx tsc --noEmit`
- [ ] **Step 7: Commit** — `style(web): standardize loading and empty states across all pages`

---

## Task 4: User Management Section

Add admin-only user management to Settings.

**Files:**
- Create: `web/src/hooks/useUsers.ts`
- Create: `web/src/pages/settings/UserManagement.tsx`
- Modify: `web/src/pages/Settings.tsx`

- [ ] **Step 1: Create useUsers.ts** — Hooks: `useUsers()` (GET /auth/users), `useCreateUser()` (POST /auth/register), `useDeleteUser()` (DELETE /auth/users/:id).
- [ ] **Step 2: Create UserManagement.tsx** — List users in a table with role badges. "Add User" form (username, password, role select). Delete button with confirmation. Only visible to admins.
- [ ] **Step 3: Update Settings.tsx** — Import and render UserManagement section (admin-only).
- [ ] **Step 4: Verify** — `cd web && npx tsc --noEmit`
- [ ] **Step 5: Commit** — `feat(web): add user management section to Settings`

---

## Task 5: Audit Log Viewer

Add audit log section to Settings for admins.

**Files:**
- Create: `web/src/hooks/useAuditLog.ts`
- Create: `web/src/pages/settings/AuditLogSection.tsx`
- Modify: `web/src/pages/Settings.tsx`

- [ ] **Step 1: Create useAuditLog.ts** — Hook `useAuditLog(limit, offset)` calling `GET /api/system/audit-log?limit=N&offset=N`.
- [ ] **Step 2: Create AuditLogSection.tsx** — Table with timestamp, action, resource type, resource ID. Simple pagination (newer/older). Filter by action type optional. Time formatted relative.
- [ ] **Step 3: Update Settings.tsx** — Import and render AuditLogSection (admin-only).
- [ ] **Step 4: Verify** — `cd web && npx tsc --noEmit`
- [ ] **Step 5: Commit** — `feat(web): add audit log viewer to Settings`

---

## Task 6: Form Validation & FeedbackBanner Polish

Improve form validation and add entrance animation to FeedbackBanner.

**Files:**
- Modify: `web/src/pages/CreateNode.tsx`
- Modify: `web/src/components/FeedbackBanner.tsx`
- Modify: `web/src/pages/NodeDetail.tsx`

- [ ] **Step 1: Update CreateNode.tsx** — Add validation: numeric fields (maxConnections, minPeers, maxPeers) must be positive integers. minPeers <= maxPeers. Show inline error messages.
- [ ] **Step 2: Update FeedbackBanner.tsx** — Add `animate-fade-in-up` class to the banner div for entrance animation.
- [ ] **Step 3: Update NodeDetail.tsx** — Fix restart button: use `SpinnerButton`, show "Restarting..." state, handle stop failure gracefully (don't attempt start if stop fails).
- [ ] **Step 4: Verify** — `cd web && npx tsc --noEmit`
- [ ] **Step 5: Commit** — `fix(web): improve form validation and feedback animations`

---

## Task 7: Network Height Display

Show network sync progress with mainnet/testnet heights in node detail.

**Files:**
- Modify: `web/src/hooks/useNodes.ts`
- Modify: `web/src/pages/NodeDetail.tsx`

- [ ] **Step 1: Add useNetworkHeight hook** — In useNodes.ts, add `useNetworkHeight()` calling `GET /api/metrics/network`. Returns `{ mainnet: number, testnet: number }`.
- [ ] **Step 2: Update NodeDetail.tsx** — In the metrics section, show "Network Height: X" and "Sync: Y%" for running nodes on mainnet/testnet.
- [ ] **Step 3: Verify** — `cd web && npx tsc --noEmit`
- [ ] **Step 4: Commit** — `feat(web): display network height and sync progress in node detail`

---

## Task 8: Final Verification

- [ ] **Step 1: Run typecheck** — `cd web && npx tsc --noEmit`
- [ ] **Step 2: Run lint** — `cd web && npm run lint`
- [ ] **Step 3: Run build** — `npm run build`
- [ ] **Step 4: Run backend tests** — `npm test`
