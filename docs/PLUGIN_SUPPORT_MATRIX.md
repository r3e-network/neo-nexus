# Plugin Support Matrix

This document provides a factual overview of plugin and configuration capabilities across all node runtime types managed by NeoNexus v4.3.1. It describes implemented behavior, not planned or aspirational features.

**Verified capability summary:**

- **NeoCli**: Managed C# DLL ZIP package installation via the Plugins page; catalog-driven launch configuration toggle for specific entries; all managed entries require restart; no hot reload is confirmed at runtime.
- **NeoGo**: No DLL ZIP installer; JSON-RPC configuration is available through the catalog; other services are built-in; adding modules requires source integration, rebuild, deploy, restart.
- **NeoRs**: Runtime Cargo features are selected in the build workspace; NeoNexus does not rebuild features; toggles in the catalog return an error that names the manual path.
- **NeoXGeth**: Built-in RPC/tooling options; no DLL package installer; source changes require rebuild/restart.
- **NeoXReth**: Build-time Reth extensions/runtime options; no dynamic installer; restart required after any change.

---

## Operative Rules

1. **Unsupported operations fail before side effects.** Package eligibility, configuration applicability, and stopped-state checks run before creating files, updating the database, or recording events.
2. **Missing or unimplemented configuration returns a clear error.** Placeholder mutation interfaces do not create `go.mod` edits, do not touch existing files, and do not report success when they cannot proceed.
3. **Valid controls remain available.** The UI shows the full set of catalog entries applicable to each runtime, including NeoGo RPC configuration.
4. **NoOp adapters fail explicitly.** Discovery and mutation methods return an actionable error rather than implying success.
5. **Repository layer enforces rules directly.** The `set_plugin_enabled` method validates node type, status, PID, and catalog applicability inside a single database transaction.

---

## Capability Table

| Node Type | Package Installer (ZIP/DLL) | Catalog Configuration | Rebuild/Source Required | Restart Required | Notes |
|-----------|----------------------------|----------------------|------------------------|------------------|-------|
| NeoCli   | ✅ Yes                     | Partial              | ❌ No                  | ✅ Yes           | DLL packages write files; catalog entries apply at next launch |
| NeoGo    | ❌ No                      | Yes (RPC only)       | ✅ Yes                 | ✅ Yes           | Adding Go modules needs source build; RPC config uses supported YAML paths |
| NeoRs    | ❌ No                      | Yes (features list)  | ✅ Yes                 | ✅ Yes           | Feature toggles fail; instructions describe manual build path |
| NeoXGeth | ❌ No                      | Limited              | ✅ Yes                 | ✅ Yes           | No managed package installer; external tooling may expose metrics |
| NeoXReth | ❌ No                      | Limited              | ✅ Yes                 | ✅ Yes           | Build-time extensions only; no runtime module loader |

---

## Node-by-Node Behavior

### NeoCli (`NodeType::NeoCli`)

**Package installation:**
- Route: POST `/plugins/install` accepts multipart `node_id`, `plugin_id`, `label`, `expected_sha256`, `package`.
- Upload validation happens as fields arrive; the target node is loaded before accepting bytes.
- Duplicate `node_id` or `package` fields are rejected immediately.
- The worker revalidates the node and installs only after checksum verification.
- Installation writes files under `<workspace>/nodes/<id>/Plugins/<plugin_id>/` and creates control metadata under `.neonexus`.
- The job record captures state, result, and file counts; the message states "package written, not confirmed loaded".

**Configuration toggle:**
- Route: POST `/plugins/<node-id>/toggle` negates the recorded state.
- The UI handler calls `ensure_plugin_configuration_supported` to validate catalog applicability.
- Repository implementation checks status and PID before updating `plugin_states`.
- Events are recorded for `PluginUpdated`.

**Constraints:**
- Package install rejects active nodes or those with a recorded PID.
- Uploads create temporary files only after target-node validation succeeds.

**Files:**
- [`src/web/plugin_ops.rs`](../src/web/plugin_ops.rs) — upload handler and job body
- [`src/plugins/manager.rs`](../src/plugins/manager.rs) — shared install eligibility checks and packaging logic
- [`src/repository/nodes_plugins/plugin_states.rs`](../src/repository/nodes_plugins/plugin_states.rs) — transactional configuration updates
- [`src/web/pages/plugins.rs`](../src/web/pages/plugins.rs) — UI presentation and toggle handler

---

### NeoGo (`NodeType::NeoGo`)

**Package installation:**
- The shared `ensure_plugin_installable` check runs before any work.
- It fails with explicit text that DLL ZIP packages are unsupported and NeoGo uses built-in services.
- The catalog lists RPC configuration, which remains functional.

**Configuration toggle:**
- Valid catalog entries (including RPC) pass `ensure_plugin_configuration_supported`.
- Invalid catalog ids return an error naming the runtime and pointing to the support matrix.
- Generic module toggles delegate to `catalog/neo_go_modules.rs` functions.

**Installation placeholder:**
- `install_module` always returns `automated source integration and rebuilding are not implemented`; no filesystem mutations occur.
- `toggle_module` always returns `generic module configuration is not implemented`; it describes missing verified mapping and directs to supported settings.

**Files:**
- [`src/catalog/neo_go_modules.rs`](../src/catalog/neo_go_modules.rs) — placeholder mutations without side effects

---

### NeoRs (`NodeType::NeoRs`)

**Package installation:**
- Fails early via `ensure_plugin_installable` because Rust binaries do not accept DLLs at runtime.

**Configuration toggle:**
- The catalog enumerates features, but the toggle path does not have the runtime's workspace context.
- `toggle_feature` returns an error stating automated feature management is unimplemented and names the manual path (edit Cargo features, rebuild, deploy, restart).
- `generate_build_instructions` reports no changes and restates the manual steps.

**Files:**
- [`src/catalog/neo_rs_features.rs`](../src/catalog/neo_rs_features.rs) — feature toggles that fail explicitly

---

### NeoXGeth (`NodeType::NeoXGeth`)

**Package installation:**
- Fails via `ensure_plugin_installable` with guidance that the runtime uses built-in extensions.

**Configuration toggle:**
- The catalog determines applicability; entries outside the runtime's managed scope are rejected.
- Operators are directed to the plugins page controls or external tools.

**Files:**
- Shared helpers in [`src/plugins.rs`](../src/plugins.rs) provide runtime-specific messages

---

### NeoXReth (`NodeType::NeoXReth`)

**Package installation:**
- Fails via `ensure_plugin_installable` and states the build-time extension model.

**Configuration toggle:**
- Same catalog-based gating as other runtimes; invalid IDs produce an actionable error.

**Files:**
- Shared helpers in [`src/plugins.rs`](../src/plugins.rs)

---

## Adapter Behavior

### NoOpPluginSystemAdapter

All three public methods fail using the crate-shared error helper `plugin_adapter_unavailable`:

- `discover_plugins`: returns an error saying no adapter was registered and supplies operator guidance.
- `install_plugin`: returns the same error; no discovery is implied.
- `toggle_plugin`: returns the same error; no default context is assumed.

**Location:**
- [`src/supervisor/model.rs`](../src/supervisor/model.rs#L1060-L1080)

### NodeManager Facade

The facade separates capability rejection from adapter lookup:

- Unsupported package capability fails first.
- If capable but the registry lacks an adapter, a separate error names the runtime and refuses the operation.
- Installation and toggle include node ID and name in errors.

**Location:**
- [`src/node_manager.rs`](../src/node_manager.rs)

### Repository `set_plugin_enabled`

The repository implementation holds a single transaction during which it:

- Queries status, PID, and node type for the requested node.
- Validates catalog applicability for the node type and plugin id.
- Ensures the node is stopped before allowing configuration changes.
- Updates the `plugin_states` table.

**Location:**
- [`src/repository/nodes_plugins/plugin_states.rs`](../src/repository/nodes_plugins/plugin_states.rs#L4-L42)

---

## Limits and Constraints

- **Package size cap:** 2 GiB enforced while streaming the upload.
- **Expanded-size cap:** 2 GiB enforced during unpacking inside the worker.
- **File count cap:** 20,000 files enforced during extraction.
- **Upload directory creation:** Deferred until after target-node validation succeeds.
- **Duplicate fields:** Multipart clients must send exactly one `node_id` and one `package` field.
- **Active/PID-bearing nodes:** Rejected for both package install and configuration toggle.
- **Hot reload:** Not claimed or implemented; messages state "not confirmed loaded" and "applies at next launch".

**Constants:**
- Defined in [`src/plugins.rs`](../src/plugins.rs#L80-L83):
  - `PLUGIN_PACKAGE_MAX_BYTES = 2 GiB`
  - `PLUGIN_PACKAGE_MAX_EXPANDED_BYTES = 2 GiB`
  - `PLUGIN_PACKAGE_MAX_FILES = 20_000`
  - `PLUGIN_CONTROL_DIR = ".neonexus"`

---

## Regression Coverage

Tests exist under `tests/domain/plugins_snapshots/`. Focus regressions on:

### Safety tests: [`tests/domain/plugins_snapshots/plugin_packages/safety.rs`](../tests/domain/plugins_snapshots/plugin_packages/safety.rs)

- **Unsupported runtime:** `plugin_package_manager_rejects_non_neo_cli_nodes` verifies that non-NeoCli runtimes fail before any files are created.
- **Checksum failure:** `plugin_package_manager_rejects_checksum_mismatch_before_publish` ensures no installation directory appears on mismatch.
- **Unsafe zip paths:** `plugin_package_manager_rejects_unsafe_zip_paths` confirms containment and no filesystem escape.

These tests confirm **no go.mod edits**, **no existing file byte changes**, and **no successful records** for failed paths.

### State tests: [`tests/domain/plugins_snapshots/plugin_packages/state.rs`](../tests/domain/plugins_snapshots/plugin_packages/state.rs)

- **Persist per-node states:** `plugin_state_can_be_enabled_and_listed_per_node` covers enabled/disabled rows and retrieval by node id.

### Web integration tests: [`tests/web.rs`](../tests/web.rs)

- **Successful upload flow:** authenticated POST, multipart builder, job submission, redirect with "install started" message.
- **Invalid toggle sequence:** attempts to toggle on an active node or with an invalid plugin id; flash includes "not changed".

### Additional regression requirements

- **NoOp failures:** `supervisor/model.rs` NoOpPluginSystemAdapter methods reject with actionable text and do not return empty success.
- **Facade rejection:** `node_manager.rs` returns capability-first errors and includes node context.
- **Catalog placeholders:** `neo_go_modules.rs` and `neo_rs_features.rs` fail without side effects.
- **Repository gating:** direct calls to `set_plugin_enabled` reject active nodes and invalid plugin ids.
- **Upload validation:** POST handlers reject duplicate fields, missing targets, and active nodes before tempfile creation.
- **Job visibility:** recent plugin jobs display state, description/details, and refresh link.
- **Success semantics:** messages avoid claiming runtime loading; they note package-written/configuration-changed outcomes.

---

## Compatibility Effects

**Behavior changes from prior drafts:**

- Operations that previously returned generic success or warnings now fail with explicit errors.
- The NoOp adapter no longer implies inventory by returning an empty list; it fails.
- Package uploads require `node_id` before `package` bytes; duplicate fields are rejected.
- Temporary directories are created after validation, reducing stray artifacts on bad input.
- Configuration toggles validate against the managed catalog; entries that do not apply are refused.
- The three exported catalog placeholder interfaces (`install_module`, `toggle_module`, `toggle_feature`) never mutate files when they cannot proceed.

**Backward compatibility notes:**

- Public signatures preserved for adapters and handlers.
- Clients relying on success codes for unsupported operations will receive errors; messages explain the manual alternatives.
- Supported configurations (NeoCLI DLL install, NeoGo RPC config) remain operational under the existing workflows.

---

## Verification Limitations

**Not performed during this task:**

- `cargo build`, `cargo fmt`, `cargo clippy` compilation passes.
- Test execution via `cargo test` harnesses.
- Browser verification of the Plugins page layouts and messages.

**Remaining tasks:**

- Independent Verify/Browser reviewers should run build, lint, and test commands.
- Manual browser testing to confirm badges, notices, tables, job panels, and flash messages match the implemented logic.
- Security review of ZIP unpacking constraints and path sanitization under high load.

---

## See Also

- [Task #122 continuation summary](../../cache/plans/节点管理_优化_收尾_f334068d.md) — session history and implementation continuity
- [`src/plugins.rs`](../src/plugins.rs) — public helpers for capability, guidance, and constants
- [`src/web/pages/plugins.rs`](../src/web/pages/plugins.rs) — UI rendering and handlers
- [`src/web/plugin_ops.rs`](../src/web/plugin_ops.rs) — upload endpoint and background job
