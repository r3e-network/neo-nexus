# Secure Signer Orchestration Implementation Plan

> **For Implementer:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extend NeoNexus secure signer support with operator-grade lifecycle orchestration: deployment helper commands, Nitro attestation and ciphertext startup flows, signer readiness checks, and node-level signer health visibility.

**Architecture:** Keep signer secrets outside NeoNexus and outside node configs. NeoNexus stores only signer metadata plus local orchestration references such as the `secure-sign-service-rs` workspace path, startup port, KMS ciphertext blob path, and AWS region. The backend exposes safe orchestration endpoints that either return copyable host commands or execute non-secret operations such as attestation fetch, recipient-ciphertext startup, and signer status checks through `secure-sign-tools`.

**Tech Stack:** TypeScript, Express, better-sqlite3, React, TanStack Query, Vitest, `secure-sign-service-rs` CLI tools and scripts

### Task 1: Orchestration Domain Model

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/core/SecureSignerManager.ts`
- Test: `tests/unit/SecureSignerOrchestration.test.ts`

**Step 1: Write the failing test**

Add tests that prove:
- orchestration metadata is normalized and persisted
- lifecycle helpers derive service/startup ports from signer endpoints
- deployment and unlock command templates are generated per signer mode

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/SecureSignerOrchestration.test.ts`
Expected: FAIL because orchestration fields and helper methods do not exist.

**Step 3: Write minimal implementation**

Add:
- orchestration metadata fields (`workspacePath`, `startupPort`, `awsRegion`, `kmsKeyId`, `kmsCiphertextBlobPath`)
- port parsing helpers
- generated command sets for software, SGX, and Nitro modes

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/SecureSignerOrchestration.test.ts`
Expected: PASS

### Task 2: Safe Tool-Driven Operations

**Files:**
- Modify: `src/core/SecureSignerManager.ts`
- Test: `tests/unit/SecureSignerOrchestration.test.ts`

**Step 1: Write the failing test**

Add tests that prove:
- readiness checks can run `secure-sign-tools status` for localhost / vsock-compatible profiles
- Nitro recipient attestation can be fetched through `secure-sign-tools recipient-attestation`
- Nitro recipient ciphertext startup can be executed without NeoNexus ever handling plaintext passphrases

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/SecureSignerOrchestration.test.ts`
Expected: FAIL because no tool execution helpers exist.

**Step 3: Write minimal implementation**

Implement injected command-runner helpers that can:
- run `status`
- run `recipient-attestation`
- run `start-recipient`
- return structured results and operator-safe errors

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/SecureSignerOrchestration.test.ts`
Expected: PASS

### Task 3: Orchestration API

**Files:**
- Modify: `src/api/routes/secureSigners.ts`
- Test: `tests/integration/secure-signers.orchestration.router.test.ts`

**Step 1: Write the failing test**

Add route tests that prove:
- `GET /api/secure-signers/:id/orchestration` returns lifecycle helpers and readiness
- `POST /api/secure-signers/:id/attestation` returns a recipient attestation document
- `POST /api/secure-signers/:id/start-recipient` accepts ciphertext and triggers safe startup

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/integration/secure-signers.orchestration.router.test.ts`
Expected: FAIL because these endpoints do not exist.

**Step 3: Write minimal implementation**

Add those endpoints and map them to `SecureSignerManager`.

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/integration/secure-signers.orchestration.router.test.ts`
Expected: PASS

### Task 4: Node-Facing Signer Health

**Files:**
- Modify: `src/core/NodeManager.ts`
- Modify: `src/api/routes/nodes.ts`
- Test: `tests/unit/NodeManager.secure-signer-health.test.ts`

**Step 1: Write the failing test**

Add tests that prove:
- nodes with secure signer bindings expose signer health metadata
- health checks defer to the bound signer profile
- standard-wallet nodes return no signer health payload

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/NodeManager.secure-signer-health.test.ts`
Expected: FAIL because node-level signer health does not exist.

**Step 3: Write minimal implementation**

Add:
- node manager helper to resolve bound signer status
- node route for signer health lookup
- minimal status structure for UI consumption

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/NodeManager.secure-signer-health.test.ts`
Expected: PASS

### Task 5: Settings And Node UX

**Files:**
- Modify: `web/src/hooks/useSecureSigners.ts`
- Modify: `web/src/pages/Settings.tsx`
- Modify: `web/src/pages/NodeDetail.tsx`

**Step 1: Write the failing test**

Prefer build-safe assertions and targeted helpers over shallow component tests. Verify:
- secure signer orchestration payloads compile
- node detail can consume signer health data
- settings page can trigger orchestration actions through hooks

**Step 2: Run test to verify it fails**

Run: `npm run build --prefix web`
Expected: FAIL if the UI and hook types are not aligned.

**Step 3: Write minimal implementation**

Add:
- orchestration hook methods
- lifecycle command display
- attestation fetch and ciphertext startup controls
- node detail signer health card with readiness state

**Step 4: Run test to verify it passes**

Run: `npm run build --prefix web`
Expected: PASS

### Task 6: Full Verification

**Files:**
- Modify: `README.md`

**Step 1: Run targeted tests**

Run: `npm test -- tests/unit/SecureSignerOrchestration.test.ts tests/integration/secure-signers.orchestration.router.test.ts tests/unit/NodeManager.secure-signer-health.test.ts`
Expected: PASS

**Step 2: Run full test suite**

Run: `npm test`
Expected: PASS

**Step 3: Run static verification**

Run: `npm run typecheck`
Expected: PASS

**Step 4: Run production build**

Run: `npm run build`
Expected: PASS

**Step 5: Update docs**

Document:
- what NeoNexus can orchestrate directly
- what still requires host-level or cloud-specific prerequisites
- why plaintext passphrase submission through the web UI is intentionally unsupported
