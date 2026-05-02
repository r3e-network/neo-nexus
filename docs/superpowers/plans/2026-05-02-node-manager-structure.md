# NodeManager Structure Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract process attachment and managed-path helper logic from `NodeManager` while preserving behavior.

**Architecture:** Add a focused `nodeProcessAttachment` helper module under `src/core`. `NodeManager` keeps orchestration and persistence, and delegates pure process/path decisions to the helper.

**Tech Stack:** TypeScript, Vitest, existing lifecycle helpers, existing `NodeInstance`/`NodeConfig` types.

---

### Task 1: Add Helper Module Tests

**Files:**
- Create: `tests/unit/node-process-attachment.test.ts`

- [ ] **Step 1: Write failing tests**

Create tests for `isValidProcessId`, `parseProcessIds`, `isPathWithinOrEqual`, `isManagedNodeDirectory`, `scoreAttachCandidate`, and `getAttachedProcessState`.

- [ ] **Step 2: Run tests to verify failure**

Run: `npx vitest run tests/unit/node-process-attachment.test.ts --reporter verbose`

Expected: fail because `src/core/nodeProcessAttachment.ts` does not exist.

### Task 2: Implement Helper Module

**Files:**
- Create: `src/core/nodeProcessAttachment.ts`

- [ ] **Step 1: Move pure logic**

Implement the helper functions using code currently private inside `NodeManager`.

- [ ] **Step 2: Run helper tests**

Run: `npx vitest run tests/unit/node-process-attachment.test.ts --reporter verbose`

Expected: all helper tests pass.

### Task 3: Delegate From NodeManager

**Files:**
- Modify: `src/core/NodeManager.ts`

- [ ] **Step 1: Replace private helpers with imports**

Use helper functions for attach state, candidate scoring, PID parsing, path containment, and managed-node classification.

- [ ] **Step 2: Run NodeManager regression tests**

Run: `npx vitest run tests/unit/NodeManager.system-actions.test.ts --reporter verbose`

Expected: all existing system-action tests pass.

### Task 4: Extract Runtime Metrics Helper

**Files:**
- Create: `src/core/nodeRuntimeMetrics.ts`
- Create: `tests/unit/node-runtime-metrics.test.ts`
- Modify: `src/core/NodeManager.ts`

- [ ] **Step 1: Extract metrics collection**

Move runtime metric collection into a helper that returns a `NodeMetrics` object. Keep persistence and event emission in `NodeManager`.

- [ ] **Step 2: Run metrics helper tests**

Run: `npx vitest run tests/unit/node-runtime-metrics.test.ts --reporter verbose`

Expected: all metrics helper tests pass.

### Task 5: Final Verification

**Files:**
- No code changes expected.

- [ ] **Step 1: Run full verification**

Run: `npm run verify`

Expected: backend and frontend lint/typecheck/tests/build pass.

- [ ] **Step 2: Run backend production build**

Run: `npm run build:backend`

Expected: TypeScript production build exits with code 0.
