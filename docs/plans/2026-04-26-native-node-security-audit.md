# NeoNexus Native Node / Secure Signer Audit

Date: 2026-04-26

## Executive summary

NeoNexus already has a solid self-hosted Neo node manager foundation: TypeScript backend, React UI, SQLite persistence, process control, config generation, plugin management, metrics, audit log, integrations, and secure-signer profile orchestration. Baseline quality is healthy: test, typecheck, and lint were passing before this audit.

The main architectural risk is that imported/native nodes are currently treated too much like NeoNexus-owned nodes. To satisfy the product goal of managing mostly unmodified upstream `neo-cli` / `neo-go`, the manager must become more non-invasive:

1. Preserve exact native binary/config paths.
2. Avoid rewriting imported configs by default.
3. Distinguish attached external processes from NeoNexus-spawned children.
4. Track which files/plugins are owned by NeoNexus vs. external/user-managed.
5. Use RPC, logs, process metadata, and minimal config overlays rather than requiring custom node forks.

Secure signer support now includes explicit capability declarations, fail-closed SignClient policy generation (`DenyByDefault`), hardware-required policy blocking for software signers, and audit-log entries for signer profile creation, testing, capability reads, policy configuration, and blocked policy decisions. The actual transaction whitelist must still be enforced inside the signer service / TEE / HSM boundary; NeoNexus now configures and audits that policy instead of claiming host-side enforcement is sufficient.

## Completed in this refactor pass

- Fixed path-boundary validation so `/home2/...` can no longer pass just because it starts with `/home`.
- Fixed exported snapshot version to read the real root `package.json` version instead of the stale `2.0.0` fallback.
- Fixed imported/native `neo-go` config file handling so `paths.config = /opt/node/protocol.yml` is used directly instead of becoming `/opt/node/protocol.yml/protocol.yml`.
- Hardened repository-attached process state:
  - `startNode()` now refuses nodes already marked `running` or `starting` in the DB, even without an in-memory child wrapper.
  - `stopNode()` can stop a DB-attached process by PID when no in-memory wrapper exists.
- Updated config writing/auditing helpers to accept either a config directory or an exact config file path for `neo-cli` and `neo-go`.
- Added imported native-node ownership modes:
  - `observe-only`: monitor/audit only; blocks config writes and process lifecycle actions.
  - `managed-config`: allows config metadata/config-file management but blocks process start/stop.
  - `managed-process`: allows NeoNexus to attach/start/stop after strict PID ownership validation.
- Added API validation for import `ownershipMode` and defaulted imports to safe `observe-only`.
- Replaced first-result `pgrep` auto-attach with multi-candidate scoring using process type, cwd, argv/config path, and expected ports.
- Added secure-signer capability declarations so software fallback, SGX, Nitro, and custom modes do not overstate implemented isolation.
- Added fail-closed SignClient policy generation with operation/contract/recipient allowlists and `RequireHardwareProtection`.
- Added signer policy enforcement guard: hardware-required policies are rejected for software fallback signers.
- Added best-effort audit trail for secure-signer profile creation/tests/capability reads/policy configuration/blocked policy decisions.
- Preserved SignClient policy fields when writing plugin configs.
- Added regression tests for the above.

## Native node compatibility gaps still to address

### P0: Complete ownership metadata persistence

Implemented application-level imported-node ownership modes:

```ts
type ImportedNodeOwnershipMode = 'observe-only' | 'managed-config' | 'managed-process';
```

Current semantics:

- `observe-only`: no filesystem mutation and no lifecycle control; monitor/audit only.
- `managed-config`: NeoNexus may update adopted config metadata/files but must not start/stop the external process.
- `managed-process`: NeoNexus may attach/start/stop after strict PID/process ownership validation.

Remaining work: promote ownership metadata into explicit DB columns/runtime metadata instead of storing it only in `settings.import`, and add an explicit detach/adopt flow in the UI/API.

### P0: Persist native binary/config metadata

Current model has `paths.config`, but not an explicit `paths.binary` / `runtime.executable` / `runtime.args`. Add fields to preserve:

- detected binary path (`neo-go`, `neo-cli`, `neo-cli.dll`)
- exact config file path
- original launch command when attaching to a running process
- whether the process is externally owned

### P0: Make imports non-mutating by default

For imported nodes, these operations must not overwrite native config/plugin files unless the user explicitly adopts full management:

- `updateNode()`
- `installPlugin()` / `uninstallPlugin()`
- `syncNodeSecureSigner()`
- config audit remediation

### P0: Process attach/detach model

Replace global `pgrep -f neo-cli|neo-go` attach with matching by PID plus one or more of:

- executable path
- cwd
- command-line `--config-file`
- bound RPC/P2P ports
- config/data path

External PIDs should support a safe `detach` operation. Killing external PIDs should require an explicit user action or a dedicated `forceExternal` flag.

### P1: Patch/overlay configs instead of regenerating full configs

For native/upstream compatibility:

- Prefer official config shipped with the selected node version.
- Apply minimal JSON/YAML patches for ports, paths, enabled plugins, and user-chosen settings.
- Preserve unknown fields and user customizations.
- Make `settings.customConfig` an explicit deep overlay.

### P1: Track plugin ownership

On import, discover plugins as external. Only remove/overwrite plugin files that NeoNexus installed. For external plugins, provide audit/diff/adopt flows.

## Secure signer/key-protection gaps still to address

Current NeoNexus secure-signer support stores signer profiles and wires `neo-cli` `SignClient` endpoint configs. The actual security boundary must be enforced by the signer service / TEE / HSM.

### P0: Policy enforcement must remain inside signer boundary

NeoNexus now emits fail-closed SignClient policy config (`DenyByDefault`) with optional:

- contract hashes
- allowed methods/operations
- recipient allowlist
- hardware-protection requirement

NeoNexus also rejects `requireHardwareProtection` for software fallback profiles. Remaining signer-side policy should cover at least:

- network magic
- account/public key
- contract hashes
- allowed methods/operations
- recipient allowlist
- asset/token allowlist
- per-transfer/per-day amount limits
- valid-until-block / nonce constraints
- node identity / caller identity

NeoNexus can manage policy, but the signer/TEE/HSM must enforce fail-closed.

### P0: Add signer identity and transport authentication

Recommended options:

- mTLS between node/plugin and signer
- pinned signer public key/certificate
- attestation-bound signer public key
- per-node ACL token scoped to one signer profile and operation policy

### P0: Add real TEE/HSM attestation verification

Nitro:

- verify attestation document chain
- verify PCRs / expected EIF measurement
- verify nonce freshness
- bind attested public key to the signer endpoint/account

SGX:

- verify quote
- pin MRENCLAVE/MRSIGNER/ISVSVN
- bind report data to signer public key/session

HSM:

- add explicit provider abstraction (PKCS#11 / AWS KMS / CloudHSM / etc.)
- store only key handles and policy references, never private material

### P1: Operational hardening

- Make secure-signer binding admin-only or add RBAC permissions.
- Prevent deleting signer profiles while nodes reference them, unless forced with cleanup.
- Resync bound node `SignClient` configs when signer endpoint/name changes.
- Audit-log secure signer CRUD, readiness, attestation fetch, recipient startup, and node binding/unbinding.
- Pass recipient ciphertext via stdin or protected temp file instead of process arguments.
- Shell-quote generated orchestration commands.

## Recommended next implementation order

1. Add `managementMode` and `paths.binary`/`paths.configFile` schema migration.
2. Refactor runtime adapters (`NeoCliNode`, `NeoGoNode`) to prefer preserved native binary/config paths.
3. Gate filesystem mutation APIs by `managementMode`.
4. Refactor config generation into official-base + minimal overlay.
5. Add plugin ownership/adoption model.
6. Add signer policy data model and signer-side enforcement integration.
7. Add attestation/identity verification and authenticated signer transport.
