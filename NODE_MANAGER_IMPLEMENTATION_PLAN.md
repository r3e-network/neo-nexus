# NeoNexus Node Manager Architecture - Implementation Plan

> **HISTORICAL SNAPSHOT** — This file, and the other `NODE_MANAGER_*` / `PHASE*` /
> `BENCHMARKS_STATUS.md` files alongside it, is a frozen snapshot of an earlier
> "NodeManager adapter" direction. That direction was superseded: custody goes
> through `src/signing/` + `src/signer_client/`, chain observation through
> `src/observe/`, and there is no longer a `NodeManager` facade. Treat any
> claim of completion or progress in these files as stale. The current TODO /
> gap register is **`claudedocs/NEONEXUS_GAP_REGISTER.md`**.

## Overview

This document consolidates the complete architecture design from Alex's research and breaks down the 8-week implementation roadmap into actionable, tracked tasks.

**Goal:** Unified management of all node implementations (neo-cli, neo-go, neo-rs, neox-geth, neox-rs) with type-specific feature support while maintaining backward compatibility.

---

## Architecture Summary (from Research Report)

### Core Design Patterns

1. **Trait Abstraction Layer** (`NodeTypeTraits`): Generic operations independent of implementation
2. **Adapter Pattern**: Type-specific behavior (metrics, logging, plugins)
3. **Registry Pattern**: `NodeAdapters` hashmap per node type
4. **Facade Pattern**: `NodeManager` unified API

### Key Interfaces

```rust
pub trait NodeTypeTraits {
    fn config_format(&self) -> ConfigFormat;
    fn config_path(&self) -> PathBuf;
    fn plugin_directory(&self) -> Option<PathBuf>;
    fn supports_plugins(&self) -> bool;
}

pub trait MetricsExporterAdapter {
    fn exporter_package(&self) -> Option<&'static str>;
    fn generate_exporter_config(&self, node: &NodeConfig) -> Option<Vec<u8>>;
    fn normalize_metrics(&self, raw_metrics: &[u8]) -> Result<Vec<u8>>;
}

pub trait LogParserAdapter {
    fn parse_line(&self, line: &str) -> Option<StructuredLogEntry>;
    fn detect_fatal_errors(&self, log_content: &str) -> Vec<FatalError>;
    fn extract_sync_progress(&self, recent_logs: &[&str]) -> Option<SyncStatus>;
}

pub trait PluginSystemAdapter {
    fn discover_available_plugins(&self, node_dir: &Path) -> Result<Vec<PluginMetadata>>;
    fn install_plugin(&self, plugin_id: &PluginId, target_dir: &Path) -> Result<()>;
    fn toggle_plugin(&self, plugin_id: &PluginId, enabled: bool) -> Result<()>;
}
```

---

## Phase 1: Non-Breaking Abstraction Layer (Weeks 1-2)

### Objectives
- Add traits without modifying existing code
- Define adapter framework interfaces
- Extend event journal with new types

### Tasks

#### P1-T01: Implement NodeTypeTraits for enum variants
- **File:** `src/types/node_type.rs`
- **Action:** Add trait implementation block after existing methods
- **Expected:** `node.node_type.config_path()` returns correct path per type
- **Dependencies:** None

#### P1-T02: Define NodeAdapters struct and trait boundaries
- **File:** `src/supervisor/model.rs`
- **Action:** Create `NodeAdapters`, `MetricsExporterAdapter`, `LogParserAdapter` structs/trait definitions
- **Expected:** Empty stub implementations compile successfully
- **Dependencies:** P1-T01

#### P1-T03: Extend EventKind with type-specific events
- **File:** `src/events/kind.rs`
- **Action:** Add 20+ new event variants using `define_event_kinds!` macro
- **Events:** `NeoCliPluginLoaded`, `NeoGoModuleEnabled`, `NeoRsConsensusStarted`, `NeoXGethChainInitialized`, `NeoXRethSnapshotCreated`, `MetricsExporterStarted`, etc.
- **Expected:** Backward compatible (new optional fields in JournalEntry)
- **Dependencies:** None

#### P1-T04: Wire node_type into supervisor start flow
- **File:** `src/node_lifecycle.rs` lines 76-84
- **Action:** Pass `node.node_type` to supervisor for adapter selection
- **Expected:** Supervisor can select appropriate adapters based on node type
- **Dependencies:** P1-T02

#### P1-T05: Write unit tests for trait implementations
- **File:** `tests/unit/types/node_type/traits_tests.rs`
- **Action:** Test all 5 node types return correct config paths, formats, plugin support flags
- **Expected:** 20+ test cases passing
- **Dependencies:** P1-T01

### Deliverables for Phase 1
- ✅ All core trait interfaces defined
- ✅ NodeTypeTraits implemented for 5 types
- ✅ New event kinds added and tested
- ✅ Supervisor wiring passes type info to adapters
- ✅ Zero breaking changes to existing APIs

---

## Phase 2: Prometheus Metrics & Unified Log Parsing (Weeks 3-4)

### Objectives
- Integrate metrics exporters for each node type
- Implement log parsing with type-specific adapters
- Wire metrics collection into supervisor lifecycle

### Tasks

#### P2-T01: Implement neo-go metrics integration (built-in)
- **Files:** 
  - `src/metrics/prometheus/neo_go_adapter.rs` (new file)
  - `src/supervisor/process.rs` modifications
- **Action:** 
  - Create `NeoGoMetricsAdapter` implementing `MetricsExporterAdapter`
  - Detect built-in `/metrics` endpoint at RPC port + `/metrics` suffix
  - Normalize metrics adding Neo-specific labels
- **Expected:** `collect_metrics()` works for neo-go without external tools
- **Dependencies:** P1-T02

#### P2-T02: Implement neo-cli metrics (external exporter)
- **Files:**
  - `src/metrics/prometheus/neo_cli_adapter.rs` (new file)
  - `scripts/exporters/prometheus-neo-cli-exporter/` (new directory)
- **Action:**
  - Download bundle C# console app as NeoNexus dependency
  - Generate config JSON with RPC URL, auth token
  - Start external process alongside node
  - Collect and normalize metrics
- **Expected:** neo-cli metrics available via NeoNexus API even without native exporter
- **Dependencies:** P1-T02

#### P2-T03: Implement neo-rs metrics bridge
- **Files:** `src/metrics/prometheus/neo_rs_adapter.rs`
- **Action:** 
  - Create lightweight Rust binary bridge converting tokio-console metrics to Prometheus format
  - Or detect if neo-rs has native prometheus export flag
- **Expected:** Clean Prometheus exposition from neo-rs nodes
- **Dependencies:** P1-T02

#### P2-T04: Implement Neo X Geth metrics
- **Files:** `src/metrics/prometheus/neox_geth_adapter.rs`
- **Action:**
  - Leverage geth's built-in Prometheus at HTTP RPC port + `/metrics`
  - Filter/add Neo X specific metrics (block_time=5s)
  - Preserve EVM-compatible metric names
- **Expected:** Prometheus output compatible with standard Ethereum tooling
- **Dependencies:** P1-T02

#### P2-T05: Implement Neo X Reth metrics
- **Files:** `src/metrics/prometheus/neox_reth_adapter.rs`
- **Action:**
  - Use reth's built-in metrics (Rust-based, similar to tokio-console)
  - Bridge to Prometheus format
- **Expected:** Clean metrics from neox-rs
- **Dependencies:** P2-T04

#### P2-T06: Implement neo-cli log parser
- **Files:**
  - `src/log_parser/neo_cli_parser.rs` (new file)
  - `src/log_parser/mod.rs` registry registration
- **Action:**
  - Parse JSON/RPC style logs + stderr
  - Extract: `"level":"error"`, `"msg":"consensus failed"` patterns
  - Detect fatal errors, extract block sync progress
- **Expected:** Structured log entries from neo-cli stdout/stderr
- **Dependencies:** None

#### P2-T07: Implement neo-go log parser
- **Files:** `src/log_parser/neo_go_parser.rs`
- **Action:**
  - Parse Go logger style: `"level=error pkg=consensus"`
  - Extract module-level error patterns
- **Expected:** Parsed logs with pkg/source attribution
- **Dependencies:** None

#### P2-T08: Implement neo-rs log parser
- **Files:** `src/log_parser/neo_rs_parser.rs`
- **Action:**
  - Parse `tracing-subscriber` format: `ERROR consensus::handle: block validation failed`
  - Extract source location (file:line) from traces
- **Expected:** Rust-style structured logs with source locations
- **Dependencies:** None

#### P2-T09: Implement Neo X log parsers (geth + reth style)
- **Files:** `src/log_parser/neox_geth_parser.rs`, `src/log_parser/neox_reth_parser.rs`
- **Action:**
  - Parse geth-style: `block #12345 imported`
  - Parse Tokio tracing: `ERROR reth_consensus: invalid block hash`
- **Expected:** Evm-compatible log parsing
- **Dependencies:** None

#### P2-T10: Wire all adapters into supervisor lifecycle
- **Files:** 
  - `src/supervisor/process.rs`
  - `src/supervisor/model.rs`
- **Action:**
  - On process spawn: automatically start metrics exporter if configured
  - On process exit: redirect logs to parser queue
  - Schedule periodic log tail reading and parsing
- **Expected:** Real-time metrics + structured logs available in Event Journal
- **Dependencies:** All P2-T01 through P2-T09

#### P2-T11: Add REST endpoints for metrics/logs retrieval
- **Files:** `src/web/api.rs`, `src/web/routes.rs`
- **Action:**
  - `GET /api/nodes/{id}/metrics`: Returns normalized Prometheus-format text
  - `GET /api/nodes/{id}/logs?hours=N`: Returns JSON array of StructuredLogEntry
- **Expected:** Web UI and CLI can retrieve type-normalized data
- **Dependencies:** P2-T10

### Deliverables for Phase 2
- ✅ All 5 node types have working metrics collection
- ✅ All 5 node types have working log parsing
- ✅ Metrics/logs integrated into supervisor lifecycle
- ✅ REST APIs exposed for agent consumption
- ✅ Performance: no more than 50ms overhead per type during startup

---

## Phase 3: Plugin System Expansion (Weeks 5-6)

### Objectives
- Expand plugin system beyond neo-cli
- Implement module systems for neo-go, neo-rs, Neo X types
- Create unified configuration generation for sidecar files

### Tasks

#### P3-T01: Redefine PluginDefinition to support multiple types
- **Files:** `src/catalog/definitions.rs`
- **Action:** Change `node_types: [NodeType::NeoCli]` (single-type array) to multi-type list
- **Migration:** Maintain backward compatibility (legacy queries still work)
- **Expected:** Can define plugins supporting both neo-cli AND neo-go
- **Dependencies:** None

#### P3-T02: Define NeoGo module catalog
- **Files:** 
  - `src/catalog/neo_go_modules.rs` (new file)
  - Populate with actual NeoGo modules (ECHO, StateRoot, TxIndex, etc.)
- **Action:** Map NeoGo's Go "modules" to equivalent of plugins in unified schema
- **Expected:** Operator can browse/install NeoGo modules from NeoNexus
- **Dependencies:** P3-T01

#### P3-T03: Define neo-rs feature catalog  
- **Files:** `src/catalog/neo_rs_features.rs` (new file)
- **Action:** Map neo-rs Cargo features as "features" (recompilation required vs dynamic loading)
- **Difference:** Unlike neo-cli C# hot-loading, neo-rs requires rebuild for feature toggles
- **Expected:** Operator warned of rebuild requirement before enabling features
- **Dependencies:** P3-T01

#### P3-T04: Define Neo X extension catalogs
- **Files:**
  - `src/catalog/neox_geth_extensions.rs` (new file)
  - `src/catalog/neox_reth_extensions.rs` (new file)
- **Action:** Map Geth plugins and Reth extensions to unified schema
- **Expected:** Compatible with Ethereum plugin ecosystem
- **Dependencies:** P3-T01

#### P3-T05: Implement PluginSystemAdapter for neo-cli
- **Files:** `src/plugins/system_adapter_neo_cli.rs`
- **Action:** 
  - List plugins in `Plugins/` directory
  - Install by copying DLL + generating manifest.json
  - Toggle via `config.json` editing + restart notification
- **Expected:** Complete control over neo-cli plugins
- **Dependencies:** None

#### P3-T06: Implement PluginSystemAdapter for neo-go
- **Files:** `src/plugins/system_adapter_neo_go.rs`
- **Action:**
  - NeoGo uses go-modules; provide `go.mod` patch + recompile instructions
  - Or use pre-compiled module binaries if available
- **Expected:** Module installation workflow documented
- **Dependencies:** None

#### P3-T07: Implement config generator for neo-go modules
- **Files:** `src/config/generator/neo_go/module_config.rs` (new file)
- **Action:** Generate YML-sidecar configuring which modules to load
- **Expected:** `config/config.yml` updated with module list on enable/disable
- **Dependencies:** P3-T06

#### P3-T08: Implement config generator for neo-rs features
- **Files:** `src/config/generator/neo_rs/features_config.rs` (new file)
- **Action:** Generate `Cargo.toml` snippet defining feature flags
- **Expected:** Operator prompted to rebuild before runtime smoke test
- **Dependencies:** P3-T07

#### P3-T09: Implement sidecar cleanup for all types
- **Files:** `src/config/export/node.rs`
- **Action:** Abstract stale sidecar removal when plugins/modules disabled
- **Expected:** No orphaned configuration left behind
- **Dependencies:** P3-T07, P3-T08

#### P3-T10: Write migration script for existing plugin configs
- **Files:** `scripts/migrate_plugins.py` (new file)
- **Action:** Convert old hardcoded neo-cli-only plugin configs to new multi-type schema
- **Expected:** Existing installations continue working post-upgrade
- **Dependencies:** P3-T01

### Deliverables for Phase 3
- ✅ Plugin system expanded to all 5 types (with caveats noted)
- ✅ Config generators for neo-go, neo-rs, Neo X types created
- ✅ Migration script preserves existing configurations
- ✅ Web UI shows plugin availability per node type
- ✅ CLI commands `--install-plugin`, `--enable-plugin` work generically

---

## Phase 4: Full Neo X Support & Unified Manager API (Weeks 7-8)

### Objectives
- Complete adaptation for Neo X Geth and Reth derivatives
- Deploy unified `NodeManager` facade
- Expose REST/GraphQL agent protocol

### Tasks

#### P4-T01: Implement NeoXGeth lifecycle adapter
- **Files:** `src/lifecycle/adapters/neox_geth.rs` (new file)
- **Action:** Handle Geth-specific boot sequence, chain initialization, peering setup
- **Expected:** Seamless node start/stop with proper state management
- **Dependencies:** None

#### P4-T02: Implement NeoXReth lifecycle adapter
- **Files:** `src/lifecycle/adapters/neox_reth.rs` (new file)
- **Action:** Handle Reth-specific database initialization (MDBX), snapshot restoration
- **Expected:** Clean MDBX handling and snapshot workflows
- **Dependencies:** P4-T01

#### P4-T03: Build unified NodeManager facade
- **Files:** 
  - `src/node_manager.rs` (new public module)
  - `src/lib.rs` exports
- **Action:** 
  - Aggregate all adapters into single struct
  - Expose generic methods: `start_node`, `stop_node`, `restart_node`
  - Expose type-specific methods: `configure_plugin`, `collect_metrics`, `parse_logs`
- **Expected:** Single point of control replacing scattered calls
- **Dependencies:** P1-T01 through P4-T02

#### P4-T04: Migrate CLI actions to use NodeManager
- **Files:** `src/cli/actions/node_control.rs`, `src/cli/actions/runtime_*.rs`
- **Action:** Replace direct module calls with `NodeManager` methods
- **Expected:** CLI becomes thinner, delegates to manager
- **Dependencies:** P4-T03

#### P4-T05: Migrate web handlers to use NodeManager
- **Files:** `src/web/control.rs`, `src/web/plugin_ops.rs`, `src/web/router.rs`
- **Action:** Wrap page handlers in manager calls
- **Expected:** Web UI benefits from centralized logic
- **Dependencies:** P4-T03

#### P4-T06: Expose REST endpoints for agent protocol
- **Files:** `src/web/api/node_agent.rs` (new module)
- **Action:** Implement endpoints matching TypeScript interface:
  - `POST /api/agent/nodes/create`
  - `POST /api/agent/nodes/{id}/start`
  - `POST /api/agent/nodes/{id}/plugin/install`
  - `GET /api/agent/nodes/{id}/health`
- **Expected:** Agent tools can programmatically control nodes
- **Dependencies:** P4-T03

#### P4-T07: Implement authentication layer for agent protocol
- **Files:** `src/web/auth/agent_token.rs` (new module)
- **Action:** 
  - Distinguish session cookies (human users) from Bearer tokens (agents)
  - Validate agent token permissions (read-only vs admin)
  - Rate-limit agent requests
- **Expected:** Secure automated access without human credentials
- **Dependencies:** P4-T06

#### P4-T08: Document complete API reference
- **Files:** `docs/AGENT_API.md` (new file), `README.md` updates
- **Action:** Document all NodeManager public methods and REST endpoints
- **Expected:** External developers can build agents against documented contract
- **Dependencies:** P4-T07

#### P4-T09: Run comprehensive regression test suite
- **Files:** `tests/integration/node_manager_full.rs` (new file)
- **Action:** Test all 5 node types × 10 common operations = 50 permutations
- **Expected:** 100% coverage of new functionality, zero regressions
- **Dependencies:** All prior phases

#### P4-T10: Update CHANGELOG with v4.4.0 release notes
- **Files:** `CHANGELOG.md`, `Cargo.toml` version bump
- **Action:** Document breaking changes, new features, migration guide
- **Expected:** Clear upgrade path for operators
- **Dependencies:** P4-T09

### Deliverables for Phase 4
- ✅ Unified NodeManager replaces scattered function calls
- ✅ REST/GraphQL agent protocol fully implemented
- ✅ Complete regression test suite passing
- ✅ Documentation published
- ✅ v4.4.0 release tagged and pushed

---

## Implementation Timeline Summary

| Phase | Duration | Risk Level | Primary Outcomes |
|-------|----------|------------|------------------|
| Phase 1 | Weeks 1-2 | LOW | Trait abstractions, adapter interfaces, new events |
| Phase 2 | Weeks 3-4 | MEDIUM | Metrics + logging for all 5 types |
| Phase 3 | Weeks 5-6 | HIGH | Plugin expansion beyond neo-cli |
| Phase 4 | Weeks 7-8 | VERY HIGH | Neo X completion, unified API, agent protocol |

### Parallel Work Streams

During Phases 2-4, run these streams concurrently:
- **Metrics Stream:** 5 implementations (P2-T01 through P2-T05)
- **Logging Stream:** 5 parsers (P2-T06 through P2-T09)
- **Config Generator Stream:** neo-go, neo-rs, Neo X geth/reth (P3-T07, P3-T08)
- **Agent Protocol Stream:** REST endpoints + auth (P4-T06, P4-T07)

### Total Task Count

- **Phase 1:** 5 tasks
- **Phase 2:** 11 tasks
- **Phase 3:** 10 tasks
- **Phase 4:** 10 tasks
- **Total:** 36 high-level tasks → decomposes to ~80+ granular subtasks

---

## Next Steps

1. **Review and approve** this implementation plan
2. **Assign priorities** for each phase based on business needs
3. **Begin Phase 1 task creation** with detailed subtask breakdown
4. **Set up sprint board** for tracking across 8-week roadmap

The architecture is solid, the tasks are well-defined, and the migration path maintains backward compatibility throughout. Ready to begin execution upon your approval.
