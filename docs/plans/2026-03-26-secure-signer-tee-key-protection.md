# Secure Signer / TEE Key Protection Implementation Plan

> **For Implementer:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a first-class secure-signer subsystem to NeoNexus so nodes can reference software or hardware-backed signing services without NeoNexus ever storing raw private keys or plaintext unlock material in node configuration.

**Architecture:** Keep the current Express + React structure, but introduce a `SecureSignerManager` backed by SQLite for signer profile metadata and validation. Persist only signer references, encrypted-wallet metadata, and public account identifiers in NeoNexus; generate `SignClient` plugin configuration from those references for `neo-cli` nodes, while leaving `neo-go` explicitly unsupported for remote TEE signing until an upstream-compatible flow exists.

**Tech Stack:** TypeScript, Express, better-sqlite3, React, TanStack Query, Vitest, Neo `SignClient` plugin conventions, `secure-sign-service-rs` profile model

### Task 1: Secure Signer Domain Model

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/database/schema.ts`
- Create: `src/core/SecureSignerManager.ts`
- Test: `tests/unit/SecureSignerManager.test.ts`

**Step 1: Write the failing test**

Add tests that prove:
- signer profiles are normalized and persisted
- invalid endpoint / mode combinations are rejected
- signer health checks produce deterministic status for HTTP, HTTPS, and vsock endpoints

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/SecureSignerManager.test.ts`
Expected: FAIL because no secure signer manager or schema exists.

**Step 3: Write minimal implementation**

Add:
- secure signer types (`software`, `sgx`, `nitro`, `custom`)
- unlock mode types
- SQLite table for signer profiles and last validation status
- `SecureSignerManager` CRUD, validation, and connectivity-check logic

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/SecureSignerManager.test.ts`
Expected: PASS

### Task 2: SignClient Config Generation

**Files:**
- Modify: `src/core/ConfigManager.ts`
- Modify: `src/core/PluginManager.ts`
- Test: `tests/unit/secure-signer-config.test.ts`

**Step 1: Write the failing test**

Add tests that prove:
- `SignClient` config is generated as `{ PluginConfiguration: { Name, Endpoint } }`
- plugin-specific config from the database is respected when config files are written
- non-SignClient plugin config generation still preserves generated defaults

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/secure-signer-config.test.ts`
Expected: FAIL because SignClient config is currently `{}` and stored plugin config is ignored during file generation.

**Step 3: Write minimal implementation**

Implement:
- plugin-config-aware `writePluginConfig`
- SignClient config generation from normalized signer config
- safe merge behavior for generated defaults and persisted plugin config

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/secure-signer-config.test.ts`
Expected: PASS

### Task 3: Node Binding And Auto-Wiring

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/core/NodeManager.ts`
- Modify: `src/api/routes/nodes.ts`
- Test: `tests/unit/NodeManager.secure-signer.test.ts`

**Step 1: Write the failing test**

Add tests that prove:
- secure signer bindings are stored in node settings as references / metadata only
- creating or updating a `neo-cli` node with secure signer protection auto-installs or updates `SignClient`
- trying to attach a secure signer to `neo-go` returns a clear error

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/NodeManager.secure-signer.test.ts`
Expected: FAIL because node creation/update ignores signer bindings and no SignClient wiring exists.

**Step 3: Write minimal implementation**

Add:
- `keyProtection` node settings shape
- secure signer attach/apply flow in node creation and update routes
- auto-install / update SignClient config for `neo-cli`
- explicit validation for unsupported node types

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/NodeManager.secure-signer.test.ts`
Expected: PASS

### Task 4: Secure Signer API

**Files:**
- Create: `src/api/routes/secureSigners.ts`
- Modify: `src/server.ts`
- Test: `tests/integration/secure-signers.router.actual.test.ts`

**Step 1: Write the failing test**

Add route tests that prove:
- profiles can be listed, created, updated, and deleted
- `/test` returns validation results
- malformed payloads are rejected with clear 400 responses

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/integration/secure-signers.router.actual.test.ts`
Expected: FAIL because no secure signer router exists.

**Step 3: Write minimal implementation**

Add protected API routes:
- `GET /api/secure-signers`
- `POST /api/secure-signers`
- `GET /api/secure-signers/:id`
- `PUT /api/secure-signers/:id`
- `DELETE /api/secure-signers/:id`
- `POST /api/secure-signers/:id/test`

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/integration/secure-signers.router.actual.test.ts`
Expected: PASS

### Task 5: User-Facing UI

**Files:**
- Create: `web/src/hooks/useSecureSigners.ts`
- Modify: `web/src/utils/nodePayloads.ts`
- Modify: `web/src/pages/CreateNode.tsx`
- Modify: `web/src/pages/NodeDetail.tsx`
- Modify: `web/src/pages/Settings.tsx`
- Test: `tests/unit/node-payloads.test.ts`

**Step 1: Write the failing test**

Add tests that prove:
- node payload helpers serialize secure signer references correctly
- existing node settings map back into form state
- the UI can fetch and submit secure signer profile data without raw private-key fields

**Step 2: Run test to verify it fails**

Run: `npm test -- tests/unit/node-payloads.test.ts`
Expected: FAIL because node form helpers do not understand secure signer bindings.

**Step 3: Write minimal implementation**

Add:
- secure signer CRUD hook
- settings section for managing profiles
- node create / edit controls for choosing `Standard Local Wallet` vs `Secure Signer / TEE`
- clear UX messaging that NeoNexus stores only signer references and encrypted-wallet metadata, never raw WIF

**Step 4: Run test to verify it passes**

Run: `npm test -- tests/unit/node-payloads.test.ts`
Expected: PASS

### Task 6: Full Verification

**Files:**
- Modify: `README.md`

**Step 1: Run targeted tests**

Run: `npm test -- tests/unit/SecureSignerManager.test.ts tests/unit/secure-signer-config.test.ts tests/unit/NodeManager.secure-signer.test.ts tests/integration/secure-signers.router.actual.test.ts tests/unit/node-payloads.test.ts`
Expected: PASS

**Step 2: Run full test suite**

Run: `npm test`
Expected: PASS

**Step 3: Run static verification**

Run: `npm run typecheck`
Expected: PASS

**Step 4: Run production builds**

Run: `npm run build`
Expected: PASS

**Step 5: Update documentation**

Document:
- supported secure signer modes
- `neo-cli` vs `neo-go` support boundary
- how `secure-sign-service-rs` integrates with SignClient
- remaining future work for native TPM / HSM / deeper enclave orchestration
