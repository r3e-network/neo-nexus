# Node Configuration Editing Implementation Plan

> **For Implementer:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a working node-configuration editing flow in the web UI and verify the create-node flow against the same settings payload model.

**Architecture:** Reuse the existing `PUT /api/nodes/:id` backend rather than adding new routes. Add a small shared settings payload shape in the web hooks layer, expose an update mutation, and let the node detail page switch between read-only and edit modes for safe, stopped-node-only updates.

**Tech Stack:** TypeScript, Express, React, TanStack Query, Vitest, Vite

### Task 1: Route And Hook Coverage

**Files:**
- Modify: `tests/integration/nodes.routes.test.ts`
- Create: `tests/unit/node-payloads.test.ts`
- Modify: `web/src/hooks/useNodes.ts`

**Step 1: Write the failing test**

Add tests that prove:
- `PUT /api/nodes/:id` returns the updated node
- the web payload normalizer preserves name and node settings fields

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/integration/nodes.routes.test.ts tests/unit/node-payloads.test.ts`
Expected: FAIL because update route coverage is missing and payload helpers do not exist yet.

**Step 3: Write minimal implementation**

Add an update mutation hook and extract a small payload normalizer that both create and edit flows can use.

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/integration/nodes.routes.test.ts tests/unit/node-payloads.test.ts`
Expected: PASS

### Task 2: Shared Node Form State

**Files:**
- Create: `web/src/utils/nodePayloads.ts`
- Modify: `web/src/pages/CreateNode.tsx`

**Step 1: Write the failing test**

Add a payload-normalization test for:
- optional numeric settings are omitted when blank
- booleans and custom JSON survive serialization

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/node-payloads.test.ts`
Expected: FAIL because the helper does not exist.

**Step 3: Write minimal implementation**

Create a normalizer for:
- `name`
- `type`
- `network`
- `syncMode`
- `settings.maxConnections`
- `settings.minPeers`
- `settings.maxPeers`
- `settings.relay`
- `settings.debugMode`
- `settings.customConfig`

Wire the create page to use it.

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/node-payloads.test.ts`
Expected: PASS

### Task 3: Edit UI In Node Detail

**Files:**
- Modify: `web/src/pages/NodeDetail.tsx`

**Step 1: Write the failing test**

Use build-safe verification for:
- edit mode only submits valid settings
- running nodes cannot save config edits
- save success refreshes node details

**Step 2: Run test to verify it fails**

Run: `npm run build --prefix web`
Expected: Existing detail page is read-only and has no update mutation.

**Step 3: Write minimal implementation**

Add:
- edit toggle
- form fields for name and settings
- validation/error state
- save/cancel actions
- stopped-node guard

**Step 4: Run test to verify it passes**

Run: `npm run build --prefix web`
Expected: PASS

### Task 4: Full Verification

**Files:**
- Modify: `README.md`

**Step 1: Run all tests**

Run: `npm test`
Expected: PASS

**Step 2: Run static checks**

Run: `npm run typecheck`
Expected: PASS

**Step 3: Run production build**

Run: `npm run build`
Expected: PASS

**Step 4: Update README**

Mark create/edit node configuration status based on verified behavior.
