# Native Rust App

NeoNexus is a pure Rust native application.

It is intentionally a desktop application, not a browser wrapper. The UI is a
fixed-panel workbench with an inventory rail, central task workspace, property
inspector, command toolbar, menu row, and status bar. Long data sets are
handled through bounded pagination and focused filters so node operators can
work in place without scrolling through a document page.

## Run

```bash
cargo run
```

For packaging and CI smoke checks without opening the GUI:

```bash
cargo run -- --self-check
target/release/neo-nexus --version
cargo run -- --runtime-smoke neo-rs /path/to/neo-node
cargo run -- --runtime-smoke-json neo-rs /path/to/neo-node
cargo run -- --rpc-health 10332
cargo run -- --rpc-health-json 10332
target/release/neo-nexus --workspace-readiness /path/to/neonexus.db
target/release/neo-nexus --workspace-readiness-json /path/to/neonexus.db
target/release/neo-nexus --workspace-metrics /path/to/neonexus.db
target/release/neo-nexus --workspace-metrics-json /path/to/neonexus.db
target/release/neo-nexus --workspace-metrics-prometheus /path/to/neonexus.db
target/release/neo-nexus --workspace-integrity /path/to/neonexus.db
target/release/neo-nexus --workspace-integrity-json /path/to/neonexus.db
target/release/neo-nexus --source-purity /path/to/neo-nexus
target/release/neo-nexus --source-purity-json /path/to/neo-nexus
target/release/neo-nexus --source-quality /path/to/neo-nexus/src
target/release/neo-nexus --source-quality-json /path/to/neo-nexus/src
target/release/neo-nexus --source-quality /path/to/neo-nexus/tests
target/release/neo-nexus --source-quality-json /path/to/neo-nexus/tests
target/release/neo-nexus --source-quality /path/to/neo-nexus
target/release/neo-nexus --source-quality-json /path/to/neo-nexus
target/release/neo-nexus --native-ui-audit /path/to/neo-nexus
target/release/neo-nexus --native-ui-audit-json /path/to/neo-nexus
target/release/neo-nexus --ci-policy /path/to/neo-nexus/.github/workflows/ci.yml
target/release/neo-nexus --ci-policy-json /path/to/neo-nexus/.github/workflows/ci.yml
target/release/neo-nexus --alert-preview datadog "https://event-management-intake.datadoghq.com/api/v2/events?api_key=<DD_API_KEY>" critical "RPC health unreachable"
target/release/neo-nexus --alert-preview-json datadog "https://event-management-intake.datadoghq.com/api/v2/events?api_key=<DD_API_KEY>" critical "RPC health unreachable"
target/release/neo-nexus --export-readiness-report /path/to/neonexus.db /path/to/reports
target/release/neo-nexus --export-event-journal /path/to/neonexus.db /path/to/events
target/release/neo-nexus --export-event-journal /path/to/neonexus.db /path/to/events 100 warning restart
target/release/neo-nexus --export-node-configs /path/to/neonexus.db /path/to/configs
target/release/neo-nexus --export-node-configs-json /path/to/neonexus.db /path/to/configs
target/release/neo-nexus --generate-node-config neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
target/release/neo-nexus --generate-node-config-json neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
target/release/neo-nexus --validate-node-config neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
target/release/neo-nexus --validate-node-config-json neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
target/release/neo-nexus --export-backup /path/to/neonexus.db /path/to/backups
target/release/neo-nexus --export-backup-json /path/to/neonexus.db /path/to/backups
target/release/neo-nexus --import-backup /path/to/neonexus.db /path/to/neonexus-backup.json
target/release/neo-nexus --import-backup-json /path/to/neonexus.db /path/to/neonexus-backup.json
target/release/neo-nexus --validate-backup /path/to/neonexus-backup.json
target/release/neo-nexus --validate-backup-json /path/to/neonexus-backup.json
target/release/neo-nexus --validate-wallet /path/to/validator.wallet.json
target/release/neo-nexus --validate-wallet-json /path/to/validator.wallet.json
target/release/neo-nexus --import-wallet-profile /path/to/neonexus.db /path/to/validator.wallet.json validator-wallet "Validator wallet"
target/release/neo-nexus --validate-launch-pack /path/to/private-network/manifest.json
target/release/neo-nexus --launch-pack-sidecars /path/to/private-network/manifest.json
target/release/neo-nexus --launch-pack-sidecars-json /path/to/private-network/manifest.json
target/release/neo-nexus --package-release dist
target/release/neo-nexus --verify-release-package dist
target/release/neo-nexus --verify-release-package-json dist
```

## Test

```bash
cargo test --lib
cargo test --test ci_policy
cargo test --test domain
cargo test --test repository
```

The `neo-nexus` desktop binary is marked `test = false` in Cargo metadata.
Native behavior is tested through the library plus named integration targets,
which keeps filtered test runs from invoking the GUI entrypoint.

## Native Operator Flow

The application is organized around repeatable node operations:

1. Model node definitions for neo-cli, neo-go, or neo-rs.
2. Validate binaries, generated configs, ports, storage posture, and runtime
   compatibility before launch.
3. Supervise node processes with native start, stop, restart, log capture,
   watchdog, reconciliation, and resource telemetry.
4. Triage fleet work through Operations, where action queue and selected-node
   readiness filters preserve query, severity, and target workspace context.
5. Export evidence through readiness reports, event journals, support bundles,
   backups, metrics, CI policy reports, and release package verification.

neo-rs support is part of the core runtime model. NeoNexus recognizes the
`neo-node` daemon, constrains neo-rs nodes to RocksDB posture, generates and
validates TOML configs, preserves explicit `--config` overrides as review
findings, supports runtime catalog upgrades and Fast Sync snapshot catalog
entries, and carries neo-rs readiness findings through the same native
resolution handoff used by neo-cli and neo-go.

## Documentation And Handoff

The README gives the operator entry point, this document describes the native
Rust architecture and feature surface, the validation report records the gates
that must pass before release, and the benchmark notes explain the node-manager
product patterns behind the workbench. The catalog JSON examples remain valid
machine-readable import samples rather than prose documents.

## Release Packaging

After `cargo build --release`, run `target/release/neo-nexus --package-release
dist` or `make dist`. The same package and verify actions are available from
the native Settings workspace for operator-driven release handoff. The native
packager writes a platform-specific ZIP, a sidecar JSON manifest, and a
`.sha256` checksum file under `dist/`. The ZIP contains the native executable
plus `release-manifest.json` with the binary SHA-256 and byte count; the
sidecar manifest also records the archive SHA-256 for distribution
verification. `target/release/neo-nexus --verify-release-package
<dist-dir|manifest.json|archive.zip>` validates the sidecar manifest, checksum
file, archive hash, ZIP manifest, and packaged binary hash before artifacts are
handed off. Settings records `release-packaged` and
`release-package-verified` events; `--verify-release-package-json` emits the
same result as structured JSON for CI collectors and release dashboards,
including failed-verification messages with a non-zero exit code.

## Architecture

- `src/types.rs` defines Neo node models, enums, and reusable Inventory
  filtering.
- `src/repository.rs` stores node configuration, plugin state, runtime events,
  alert delivery history, remote federation profiles, automatic probe policy,
  probe history, runtime catalog profiles, trusted runtime signer profiles,
  and workspace settings in SQLite through `rusqlite`.
- `src/alerts.rs` models alert routing policy, validates provider targets,
  builds Generic/Slack/Discord/Telegram/PagerDuty/Opsgenie/Datadog Events JSON
  alert payloads, emits redacted dry-run previews, and records searchable
  delivery outcomes without exposing route secrets in delivery history.
- `src/federation.rs` normalizes remote NeoNexus base URLs, models saved
  searchable federation profiles and monitor policy, and probes the read-only
  public status/node endpoints with retained, filterable probe history.
- `src/app/shortcuts.rs` maps cross-platform native keyboard accelerators,
  menu labels, toolbar labels, and command lists to existing workspace, node
  lifecycle, selected-node restart, filter-aware node inventory navigation, and
  view-selection actions.
- `src/app/shortcuts/labels/` keeps platform display text out of command
  definitions by formatting Command/Ctrl and Option/Alt labels through one
  tested native shortcut-label layer.
- `src/backup.rs` exports and imports workspace snapshots for node
  definitions, plugin state, plugin installation inventory, remote federation
  profiles, allowlisted workspace settings, runtime catalog profiles, trusted
  runtime signer profiles, Fast Sync manifests, and recent runtime events.
- `src/events.rs` defines structured runtime event journal records.
- `src/supervisor.rs` runs native managed processes through a reusable Rust
  supervisor so node daemons, launch-pack signer sidecars, and helper processes
  can share lifecycle control, stdout/stderr log capture, graceful stop with
  force-kill fallback, restart, and exit reconciliation.
- `src/config.rs` generates, validates, and exports neo-cli JSON, neo-go YAML,
  and neo-rs TOML configuration files, including exact managed per-node config
  paths, offline config-file validation reports, bulk workspace config export
  reports, collision-safe per-node export directories, and runtime-specific
  semantic checks.
- `src/launch.rs` derives effective launch commands from node definitions and
  managed config paths, including runtime-specific config injection detection
  for neo-go and neo-rs.
- `src/argv.rs` centralizes quote-aware argv text parsing plus reversible
  argument and command display for Node Studio, launch previews, and smoke
  evidence.
- `src/source_purity.rs` enforces the native Rust source boundary by rejecting
  Node/Web manifests, frontend source files, `node_modules`, and web/frontend
  directories, plus Docker/compose and nginx web-server deployment artifacts,
  WebView/Tauri project files, WebView/Tauri Cargo dependencies, and
  WebView/Tauri lockfile packages while skipping generated build and release
  outputs.
- `src/source_quality.rs` enforces the Rust source quality boundary by
  rejecting panic-oriented development markers and document-style native layout
  containers such as scroll areas or virtual table builders in production
  source, rejecting hardcoded platform shortcut labels in production source,
  allowing assertion shortcuts and exact shortcut-label expectations in tests,
  and rejecting all Rust files over the 200-line professional module budget.
  Repository-root scans also reject JSON, Markdown, TOML, YAML, and named
  maintenance files such as `Makefile`, `LICENSE`, and `NOTICE` over 1000
  lines with case-insensitive matching, then emit text/JSON evidence with Rust
  and maintenance-file scan counts, source snippets, and remediation hints
  without relying on external search tools.
- `src/native_ui.rs` enforces the native application shell by requiring
  eframe/egui dependencies, eframe startup, minimum window sizing, fixed
  top/bottom/left/right/central panels, explicit workspace tabs, and by
  rejecting WebView/Tauri/Wry or document-style scrolling UI markers.
- `src/ci_policy.rs` enforces the cross-platform native CI boundary by
  requiring Ubuntu, macOS, and Windows workflow coverage, source purity, source
  quality, native UI audit gates, neo-rs runtime/config checks, release
  packaging checks, and no frontend, Node, Tauri, or WebView workflow tooling.
- `src/catalog.rs` defines the plugin catalog and filterable registry model.
- `src/plugins.rs` installs SHA-256 verified neo-cli plugin ZIP packages into
  stopped nodes' managed work directories with safe extraction and manifest
  writing.
- `src/roles.rs` defines runtime role presets and private network topology
  plans.
- `src/private_network.rs` exports and validates private network launch packs
  with managed configs, deterministic seed lists, schema v10 manifests, wallet
  provisioning evidence, signer sidecar command-template expansion, no-shell
  argv execution plans, and start order.
- `src/wallet.rs` validates operator-provided encrypted Neo wallet files with
  NEP-6 structure checks, Base58Check address decoding, NEP-2 key shape checks,
  scrypt parameter checks, address-to-contract-script hash consistency,
  single-signature contract public key extraction, plaintext secret-material
  rejection, and metadata-only wallet profile import.
- `src/runtime.rs` imports local and signed HTTPS runtime release catalogs,
  selects compatible Neo runtime releases, downloads HTTPS runtime packages
  with HTTPS-only redirects, size limits, and atomic cache publishing, verifies
  SHA-256 local runtime packages, optionally enforces detached Ed25519
  signatures, installs them into the managed workspace, filters runtime
  release and installation registries, and derives node runtime upgrade plans.
- `src/app/runtime_upgrade.rs` owns the native Runtime Manager actions for
  runtime package install/download, catalog release application, selected
  running-node upgrade restarts, fleet catalog rollouts, and scheduled
  runtime upgrade policy execution so `src/app.rs` does not keep absorbing
  runtime orchestration code.
- `src/app/operations_flow/` owns non-visual Operations behavior for action
  queue filtering and focus actions, selected-node readiness filtering and
  resolution handoff, network port matrix focus, event journal
  selection/export/pruning, workspace backup validation/import safety, and
  evidence report generation.
- `src/app/operations_flow/resolution.rs` keeps the diagnostic resolution
  target to native workspace mapping in one place, so the fleet action queue
  and selected-node readiness detail open the same Config, Logs, Monitor, Node
  Studio, Operations, Plugins, Roles, Runtimes, or Wallets target.
- `src/app/views/operations/` keeps the fixed native Operations workspace split
  into focused action queue, readiness, port matrix, event journal, metrics,
  and workspace safety panels instead of returning to a monolithic or scrolling
  document surface.
- `src/app/health_events.rs` owns RPC health and remote federation status
  event severity plus duplicate-status suppression.
- `src/app/sidecar_health.rs` owns signer sidecar execution policy checks,
  endpoint health summaries, and launch-pack validation severity helpers.
- `src/app/tests.rs` keeps app-level behavior coverage beside the application
  module without letting `src/app.rs` absorb another large inline test block.
- `src/preflight.rs` resolves runtime commands on disk or through PATH,
  checks host executability, recognizes neo-cli direct or `dotnet Neo.CLI.dll`
  commands, neo-go binaries, and neo-rs `neo-node` binaries, and produces
  launch blockers plus operator-readable review findings.
- `src/runtime_smoke.rs` runs bounded runtime probe commands, captures
  stdout/stderr without pipe deadlocks, enforces a timeout, and classifies
  neo-cli, neo-go, and neo-rs responses while recording runtime binary
  path/size/SHA-256 evidence for CLI and Node Studio verification.
- `src/release_pack.rs` packages the native executable and verifies release
  manifests, checksums, archive contents, and packaged binary hashes.
- `src/rpc_health.rs` probes Neo JSON-RPC endpoints with `getversion` and
  `getblockcount`, summarizes version/block height, and classifies
  healthy/degraded/unreachable states for CLI and Node Studio checks.
- `src/snapshots.rs` imports local and signed HTTPS Fast Sync snapshot
  catalogs, verifies Fast Sync snapshot checksums, downloads HTTPS snapshots
  with HTTPS-only redirects and size limits, writes verified local cache files,
  filters catalog/registry entries, and imports raw or archived snapshot
  packages into stopped nodes' managed data directories.
- `src/dashboard.rs` builds dashboard summary projections.
- `src/diagnostics.rs` evaluates node readiness, launch readiness, generated
  config validity, configured and live IPv4/IPv6 localhost port safety,
  lifecycle state, plugin alignment, and the native workspace resolution
  target for each finding.
- `src/logs.rs` reads bounded per-node log tails, filters retained output, and
  diagnoses common startup failures with operator recommendations.
- `src/readiness_report.rs` writes timestamped text and JSON workspace
  readiness evidence for the desktop Operations panel and headless CLI,
  including stable resolution keys, labels, action labels, and operator hints.
- `src/support_bundle.rs` combines readiness, read-only integrity, metrics
  text/JSON/Prometheus snapshots, bounded event journal, redacted node
  inventory, redacted runtime log diagnosis, privacy note, SHA-256 file
  manifest, and ZIP archive output for desktop Operations and headless
  diagnostics handoff without exporting secrets, raw runtime logs, raw
  workspace databases, API keys, bearer tokens, passphrases, mnemonics, or
  seeds.
- `src/redaction.rs` centralizes diagnostics redaction for node argv,
  event-journal messages, and runtime log excerpts.
- `src/workspace_integrity.rs` performs read-only SQLite integrity,
  foreign-key, schema, index, and row-count checks for headless workspace
  verification.
- `src/event_journal_report.rs` writes timestamped text and JSON redacted
  runtime event audit evidence for headless compliance and operations
  archives.
- `src/metrics.rs` collects cross-platform system and managed process resource
  snapshots through `sysinfo`.
- `src/app.rs` owns application state, actions, and the `eframe` lifecycle.
- `src/app/views.rs` dispatches fixed workbench tabs.
- `src/app/views/` separates the native shell, operations summary, alert
  routing, node studio, resource monitor, settings, runtime manager, wallet
  profile manager, role planner, fast sync, plugin manager, configuration
  preview, and runtime log viewer.
- `src/app/widgets.rs`, `theme.rs`, `paging.rs`, `text.rs`, `draft.rs`, and
  `view.rs` hold reusable UI primitives and tested support logic, including
  native draft-to-argv conversion.

The UI is a desktop application workbench: native menu row, command toolbar,
left inventory panel, central task workspace, right property inspector, and
bottom status bar. Long collections are paginated inside fixed panels instead
of rendered as a scrolling document.
Workspace, View, and Node menus plus the fixed New Node, Reload, Start, Stop,
and Restart toolbar buttons dispatch through the same command layer as
keyboard accelerators, so desktop actions remain discoverable and testable
instead of being scattered across view-specific callbacks.

Runtime data is stored under the platform data directory by default, or under
`NEONEXUS_DATA_DIR` when that environment variable is set.

## Current Feature Surface

- Creating, editing, and deleting local node records.
- Native Federation workspace for saved remote NeoNexus public endpoint
  profiles, URL normalization, status/query-filtered profile listing, fixed
  endpoint inspection, manual probes, automatic policy-driven probes, retained
  status/query-filtered paged probe history, pruning, and audit events.
- Native Alerts workspace for disabled-by-default provider routing,
  Generic/Slack/Discord/Telegram/PagerDuty/Opsgenie/Datadog Events payload
  adapters, severity thresholds, HTTPS/localhost endpoint validation, Telegram
  Bot API `chat_id` validation, PagerDuty Events API v2 `routing_key`
  validation, Opsgenie Alert API v2 `api_key` validation with `GenieKey`
  delivery headers, Datadog Events intake v2 `api_key` validation with
  `DD-API-KEY` delivery headers, native and headless redacted route previews
  for operator and CI dry-runs, masked secret entry, background delivery,
  status/query-filtered delivery history, pruning, and policy audit events.
- Native Wallet Profiles workspace for encrypted NEP-6 wallet metadata import,
  usage/query-filtered saved profile registry, selected profile inspection,
  last-used marking, Roles signer-reference application, deletion, and
  `neo-wallet-profile-*` audit events while keeping private keys, passwords,
  and copied wallet bytes out of SQLite.
- Headless `--version`, `--help`, `--self-check`, `--runtime-smoke`,
  `--runtime-smoke-json`, `--rpc-health`, `--rpc-health-json`,
  `--workspace-readiness`, `--workspace-readiness-json`,
  `--workspace-metrics`, `--workspace-metrics-json`,
  `--workspace-metrics-prometheus`,
  `--workspace-integrity`, `--workspace-integrity-json`,
  `--source-purity`, `--source-purity-json`, `--source-quality`,
  `--source-quality-json`, `--native-ui-audit`, `--native-ui-audit-json`,
  `--ci-policy`, `--ci-policy-json`, `--alert-preview`,
  `--alert-preview-json`,
  `--export-readiness-report`, `--export-event-journal`,
  `--export-node-configs`, `--export-node-configs-json`,
  `--generate-node-config`, `--generate-node-config-json`,
  `--validate-node-config`, `--validate-node-config-json`, `--export-backup`,
  `--export-backup-json`, `--import-backup`, `--import-backup-json`,
  `--validate-backup`, `--validate-backup-json`, `--validate-wallet`, and
  `--validate-wallet-json`, and `--import-wallet-profile` commands for
  packaging, CI smoke verification, fleet
  readiness, workspace integrity, event audit
  export, bulk node config handoff, standalone node config generation, offline
  node config validation, backup automation, restore automation, and
  machine-readable operational evidence without opening a GUI window.
- Persisting node configuration in SQLite.
- Listing nodes in a fixed native inventory panel with status and search
  filtering.
- Tracking runtime version intent as `latest` or a pinned version label.
- Importing local runtime release catalog JSON and signed HTTPS runtime
  release catalogs, selecting latest host-compatible releases for neo-cli,
  neo-go, and neo-rs, filling the native install/download form from the
  selected release, downloading runtime packages from HTTPS URLs with
  HTTPS-only redirects, size limits, and atomic local cache publishing,
  installing SHA-256 verified local runtime packages into the native workspace,
  optionally enforcing detached Ed25519 signatures, persisting runtime catalog
  source profiles, trusted signer profiles, and runtime inventory in SQLite,
  planning selected-node and fleet managed catalog upgrades,
  downloading/installing the recommended release, applying compatible
  installed runtimes to stopped nodes, and applying selected or fleet running
  node catalog upgrades through restart readiness plus supervised process
  replacement with stopped-direct, running-rollout, blocked-active, and
  current/unavailable completion, no-op, and blocked-attempt event-journal
  summaries, including the failed node when an interrupted batch stops.
- Persisting a runtime upgrade policy from Settings that binds to a saved
  catalog profile, enforces signed catalogs by default, checks on a configured
  interval, rolls stopped and running fleet-node upgrades through the safe
  runtime path, caps each run's batch size,
  limits scheduled runs to optional UTC maintenance windows, spaces upgrade
  waves with an optional delay, and records policy update/run events with
  ready and planned stopped/running rollout breakdowns.
- Starting, stopping, and restarting configured node binaries from fixed
  native controls.
- Parsing and displaying Node Studio runtime arguments through a no-shell argv
  vector with quoted-value support, so paths with spaces remain one argument,
  command previews stay unambiguous, and unterminated quotes are rejected
  before launch.
- Assigning conflict-free Neo RPC/P2P/WS port blocks from Node Studio for new
  drafts or stopped selected nodes, avoiding existing node bindings and occupied
  IPv4/IPv6 localhost TCP ports while auditing persisted repairs.
- Stopping managed Unix processes with SIGTERM first, a bounded grace period,
  structured stop logging, and force-kill fallback for stubborn children.
- Previewing runtime-specific launch commands before starting processes.
- Capturing launched process stdout/stderr into stable per-node log files.
- Viewing captured logs in a bounded, paginated native workspace.
- Searching captured logs and following the retained tail window without
  switching to a document-style layout.
- Starting, stopping, and restarting the selected node from the toolbar, Node
  menu, Summary, inspector, or keyboard accelerator, plus restart handling
  after a previously supervised child process has exited.
- Reconciling child process exits, stale runtime state from previous app
  sessions, and operator-confirmed missing PID records from Monitor.
- Automatically restarting abnormal exits with bounded exponential backoff.
- Persisting and editing watchdog enablement, retry attempts, base delay, and
  maximum retry delay from a native Settings workspace.
- Monitoring host CPU and memory pressure plus per-node PID CPU, resident
  memory, uptime, missing-process telemetry, manual telemetry refresh, and
  missing-process repair, with one-click focus from Telemetry health into the
  managed-process table's Missing filter.
- Recording lifecycle, restart, plugin, config, log, backup, and watchdog
  activity in a structured runtime event journal.
- Filtering runtime events by severity and text, reporting match counts,
  selecting full event details in the native Operations workspace while
  synchronizing node-event selection with the active node, exporting
  timestamped redacted text/JSON event audit evidence with an
  `event-journal-exported` audit record, exposing the same export headlessly
  with optional limit/severity/query filters, and pruning the journal to a
  bounded recent history.
- Runtime binary preflight for Node Studio, Operations readiness, and launch
  protection. The application resolves PATH commands, checks Unix executable
  bits where available, supports neo-cli direct binaries and `dotnet
  Neo.CLI.dll` wrapper launches, recognizes neo-go command binaries, and
  recognizes the neo-rs `neo-node` daemon expected by the Rust node.
- Runtime smoke testing from Node Studio and the headless CLI. Smoke probes use
  short `--version` / `--help` style invocations, kill timed-out processes, and
  classify responses as passed, review, failed, timed-out, or blocked. The
  `--runtime-smoke-json` mode emits the same preflight, runtime binary hash,
  attempt, timeout, and captured-output evidence with stable JSON status labels,
  redacted command lines and captured output, and non-zero exit codes for
  blocked or failed probes. CI policy requires the neo-rs smoke path to assert
  passed text output and verified JSON binary evidence, including SHA-256.
- JSON-RPC health testing from Node Studio and the headless CLI. Health probes
  call `getversion` and `getblockcount`, then classify the endpoint as healthy,
  degraded, or unreachable without requiring a browser shell. Selected
  node checks are persisted in SQLite and surfaced as latest health/height in
  the Summary and Node Studio panels, including recent H/D/U trend summaries
  and bounded per-node retention controls in Settings. The
  `--rpc-health-json` mode emits structured method-level evidence and returns a
  non-zero exit code for degraded or unreachable endpoints.
- Automatic RPC health monitoring for running nodes uses background threads and
  a native result channel so network probes do not block the egui frame loop.
  Repeated same-status checks update history without flooding the event
  journal; first checks and status changes are recorded as operational events.
  The monitor enablement and interval are persisted in SQLite workspace
  settings and policy changes are audit logged from the native Settings panel.
- Alert routing uses the same non-blocking pattern: matching warning or
  critical events are delivered on a background thread with provider-specific
  payloads, including Telegram Bot API `sendMessage` JSON, PagerDuty Events API
  v2 `trigger` JSON, Opsgenie Alert API v2 JSON with `GenieKey` delivery
  headers, and Datadog Events intake v2 JSON with `DD-API-KEY` delivery headers.
  `--alert-preview` and `--alert-preview-json` reuse the same provider request
  builder without sending a network request, then emit redacted endpoint,
  header, and payload evidence for CI and operator review. Delivery outcomes
  are written to SQLite, and the status bar exposes pending sends.
- Tracking process status and PID in the local database.
- Managing neo-cli, neo-go, and neo-rs node definitions.
- Runtime-compatible storage engine, RPC, P2P, and WebSocket port settings,
  with neo-go constrained to LevelDB, neo-rs constrained to RocksDB, and
  per-node ports required to be non-zero and distinct in the native
  draft/repository path.
- Dashboard summary cards.
- Built-in plugin catalog with state/category/query-filtered native registry
  and per-node plugin enablement state.
- neo-cli plugin ZIP package installation into `nodes/<id>/Plugins/<PluginId>/`
  for stopped nodes, with SHA-256 verification, safe native ZIP extraction,
  replacement staging, manifest writing, SQLite installation inventory, and
  plugin-installed audit events.
- Role presets for RPC/API, State, Indexer, Consensus, and Observer node
  postures.
- neo-cli role application that updates plugin state and records role-applied
  audit events, while neo-go and neo-rs plans stay tied to runtime-managed
  configuration posture.
- Private network topology planning for one, four, and seven node layouts
  across neo-cli, neo-go, and neo-rs.
- Private network template materialization into local node definitions, using
  an existing same-runtime node as the binary/version source and applying
  neo-cli role plugin state during creation.
- Native signer handoff preview in the Roles workspace that parses committee
  keys and signer reference lines into signer, wallet, endpoint, sidecar,
  no-shell argv plan counts, and supervisor-ready sidecar process specs before
  launch pack export; exported launch pack manifests can be reloaded into the
  same specs for native supervision, started, stopped, audited, and
  watchdog-restarted from the Roles workspace, with HTTP reachability checks
  for signer endpoints, plus bundled-only sidecar execution by default with a
  persisted native `Allow External` operator override for PATH or pack-external
  signer binaries, and audited policy changes plus blocked starts, and
  `--launch-pack-sidecars[-json]` exposes the recovered specs for operator
  scripts and CI handoff. Selected Wallet Profiles can inject contract public
  key plus encrypted wallet path as signer references before export. Text
  sidecar handoff evidence and supervisor log command headers redact sensitive
  display-command values while preserving structured argv execution plans.
- Private network launch pack export for stopped materialized plans, including
  deterministic network magic, validator count, seed list, operator-supplied
  committee public keys, signer wallet/HTTP(S) endpoint references, optional
  signer sidecar command templates with `{wallet}`, `{endpoint}`, `{label}`,
  and `{public_key}` expansion, no-shell argv execution plans, per-node managed
  configs, schema v10 launch manifest, `wallet-provisioning.json`,
  `wallets/README.md`, start order,
  generated Unix/macOS shell plus Windows PowerShell
  preflight/start/health/stop scripts, startup guards for missing binaries,
  configs, work directories, live PID files, occupied local ports, encrypted
  NEP-6/NEP-2 signer wallet validation, signer command validation, signer
  sidecar PID files, and post-start signer, node PID, port, and JSON-RPC health
  probes. Launch packs explicitly exclude private keys and passwords.
- Fast Sync snapshot manifests with local source paths or HTTPS URLs,
  local and signed HTTPS snapshot catalogs, catalog-to-manifest conversion,
  runtime/network/query-filtered catalog and registry views, verification/cache
  state filters, HTTPS-only snapshot downloads, download size limits,
  SHA-256 verification, mismatch protection, and verified cache copies under
  the native workspace.
- Safe Fast Sync import for stopped nodes that keeps raw packages under each
  node's managed `data/<network>/fastsync/` directory, unpacks `.tar`,
  `.tar.gz`, and `.zip` packages into the runtime data root, rejects unsafe
  archive paths and target conflicts, and writes an import manifest.
- Per-node working directories for launched processes, keeping relative runtime
  data paths inside the managed workspace.
- Operations readiness workspace with a paged action queue, one-click
  critical/warning focus, selected-action and selected-check resolution
  shortcuts into the matching native workspace, selected-node readiness check
  focus with selectable detail, blocked-port focus in the network port matrix,
  filterable/exportable event journal, and fixed workspace safety controls for
  backup validation, import/export, and integrity checks.
  The same resolution model is used by headless reports so desktop operators
  and automation see consistent Config, Logs, Monitor, Node Studio, Operations,
  Plugins, Roles, Runtimes, or Wallets handoff targets, and Operations queries
  can match the stable key, visible label, action label, or operator hint while
  fixed workspace selectors narrow the action queue or selected-node checks by
  target resolution workspace without clearing other facets.
- Resolution filters are intentionally facet-preserving desktop controls. A
  severity or query search can stay active while the operator narrows the list
  to remediation work for Config, Logs, Monitor, Node Studio, Operations,
  Plugins, Roles, Runtimes, or Wallets; selection recovery keeps visible rows
  and the active node synchronized after each filter change. The workspace
  menu labels and severity buttons include counts computed from the remaining
  active facets, so operators can choose a remediation area or severity bucket
  with a clear workload signal.
- Operations diagnostics and the start path share launch readiness checks for
  runtime binary preflight, managed config validation, lifecycle state, and
  active-node or IPv4/IPv6 localhost TCP listener port conflicts, so critical
  findings are launch blockers instead of cosmetic warnings. Restart readiness
  uses the same binary, config, and active-node conflict checks while
  permitting the selected running node's own current listeners before the
  managed restart sequence stops and replaces it.
- Logs workspace diagnosis highlights occupied ports, config parse failures,
  permission errors, missing files, database locks, and runtime crashes, and
  abnormal-exit notices include the diagnosis summary when a warning or
  critical pattern is present.
- Headless `--workspace-readiness <neonexus.db>` evaluates the same fleet
  diagnostics without opening the GUI, while `--workspace-readiness-json`
  emits the same gate as structured JSON for CI and operator scripts. Both
  commands exit non-zero for critical findings and include stable resolution
  keys plus the recommended native workspace/action/hint for each finding.
  `--export-readiness-report` writes timestamped `.txt` and `.json` evidence
  files with the same resolution metadata, and the same export is available
  from the fixed Operations workspace.
- Headless `--workspace-metrics <neonexus.db>` and
  `--workspace-metrics-json <neonexus.db>` expose the native metrics collector
  for scripts and monitoring integrations, including system CPU/memory,
  process counts, managed node process CPU/memory/uptime, and missing running
  PIDs. `--workspace-metrics-prometheus <neonexus.db>` exports the same
  snapshot as Prometheus text exposition with escaped labels for scrape
  pipelines.
- Headless `--workspace-integrity <neonexus.db>` and
  `--workspace-integrity-json <neonexus.db>` run read-only SQLite
  `integrity_check`, foreign-key, required table/column, required index,
  row-count, and page metadata checks without mutating the workspace. The same
  check is available from the native Operations workspace and records a
  `workspace-integrity-checked` event with pass/fail severity.
- Workspace backup export/import for node definitions, plugin state, plugin
  installation inventory, remote federation profiles, allowlisted workspace
  settings, and recent runtime catalog profiles, trusted runtime signer
  profiles, Fast Sync manifests, and recent runtime events, with imported node
  runtime state reset safely to
  stopped and unsupported setting keys rejected. The same backup export path
  is available headlessly against an existing workspace database from
  `--export-backup <neonexus.db> <output-dir>`, and
  `--export-backup-json <neonexus.db> <output-dir>` writes the backup while
  emitting structured evidence for scheduled jobs. Headless restore is
  available from `--import-backup <neonexus.db> <backup.json>` and
  `--import-backup-json <neonexus.db> <backup.json>` after validation; imports
  are refused when the target workspace still has running or starting nodes.
  The native Operations workspace can validate the latest backup before import,
  requires that current validation before enabling native import, keeps the
  last validation counts visible in Workspace Safety, and records a
  `backup-validated` audit event. The same deep backup checks are available
  headlessly from
  `--validate-backup <backup.json>` and
  `--validate-backup-json <backup.json>` before an operator imports anything,
  including duplicate ID, duplicate plugin inventory, duplicate node-port, and
  event kind/severity validation. Runtime event history messages are redacted
  during export and restored with exact-match de-duplication, while runtime
  installations and cached snapshot files and remote probe telemetry stay
  machine-local and are intentionally not copied by JSON backup.
- Headless encrypted wallet validation from `--validate-wallet <wallet.json>`
  and `--validate-wallet-json <wallet.json>` checks operator-provided Neo wallet
  files before they are referenced by signer launch packs. It validates NEP-6
  object shape, accepted wallet versions, scrypt parameters, Base58Check account
  addresses, encrypted NEP-2 account keys, contract script hex, address/script
  hash consistency, extracted single-signature contract public keys,
  default/watch account counts, and plaintext private-key, password, mnemonic,
  seed, or token markers. The text command returns a non-zero exit code on
  failed checks; JSON mode emits structured evidence for CI and handoff scripts.
- Headless wallet profile import from `--import-wallet-profile <neonexus.db>
  <wallet.json> <profile-id> <label>` reuses the same encrypted wallet
  validation, records only metadata such as source path, primary address,
  extracted contract public keys, wallet SHA-256, validation time, and account
  counts, then persists the profile in SQLite for backup/restore and future
  native authoring workflows.
- neo-cli JSON, neo-go YAML, and neo-rs TOML configuration previews, local file
  exports, and generated-config validation before managed writes.
- Headless bulk config export for every workspace node, writing configs under
  `nodes/<node-id>/` and producing timestamped text/JSON evidence with node
  runtime, port, plugin, path, and byte-count metadata.
- Headless standalone config generation for neo-cli JSON, neo-go YAML, and
  neo-rs TOML without a workspace database, with immediate semantic validation
  and text/JSON evidence for generated path, byte count, and validation status.
- Headless single-config validation for neo-cli JSON, neo-go YAML, and neo-rs
  TOML files with text/JSON evidence and non-zero exit codes for semantic drift.
- Runtime-specific managed configuration application, with neo-cli publishing
  `config.json` into the node workdir and neo-go / neo-rs using managed files
  under `nodes/<id>/config/`. Running nodes can use Apply + Restart for a
  controlled config rollout through the native supervisor, while stopped nodes
  receive the managed config directly.
- neo-go launch planning that injects managed `node --config-file <node.yml>`
  arguments unless the operator already supplied a config argument.
- neo-rs launch planning that injects the managed `--config <node.toml>` unless
  the operator already supplied a config argument.
- Fixed-panel application layout with paginated nodes, wallet profiles,
  plugins, and config previews.

See [Validation Report](native-validation.md) for local verification gates.

## Runtime Release Catalogs

Runtime Manager can import a local JSON catalog such as
`docs/runtime-catalog.example.json`, or a signed HTTPS catalog with a detached
Ed25519 signature and trusted public key. Remote catalogs require a signature
before they are parsed. Catalog entries are discovery metadata: the actual
package download still goes through the Runtime Manager's HTTPS, size-limit,
SHA-256, and atomic cache validation path.

Catalog source fields:

- `Profile`: saved source profile label persisted in SQLite.
- `Saved`: previously saved source profile selector.
- `Signer`: saved trusted signer label persisted in SQLite.
- `Trusted`: previously saved signer selector.
- `Catalog`: local path or HTTPS URL.
- `Signature`: optional local path or HTTPS URL for local catalogs; required
  for HTTPS catalogs.
- `Signer key`: base64 Ed25519 public key saved as a reusable trust anchor.
- `Catalog key`: base64 Ed25519 public key used for the detached catalog
  signature on the current load.

Saved source profiles remember catalog source, detached signature source,
trusted public key, load size limit, and the last successful load metadata.
Trusted signer profiles remember reusable Ed25519 public keys, enabled state,
creation time, and last-used time. A signer can be applied explicitly to the
catalog verification key or to the runtime package verification key so source
selection and trust selection remain separate operator decisions.
For stopped or running selected and fleet candidates, Runtime Manager can plan
the latest compatible catalog release and run a managed upgrade that downloads
the package, verifies its SHA-256, installs it into the workspace, records
runtime events, and applies the installed binary/version to node definitions.
Running nodes are checked with restart readiness before the definition changes,
then replaced through the native supervisor; starting or errored nodes remain
blocked until operators reconcile them.

## Runtime Upgrade Policy

Settings includes a native Runtime upgrade policy panel. The policy is disabled
by default. When enabled, it requires a saved runtime catalog profile, can
require a signed catalog, stores a check interval and per-run fleet-node
limit, can restrict scheduled runs to a UTC maintenance window, can space
rollout waves after a successful apply, and records last-check and last-apply
timestamps in SQLite. Due checks run inside the native application lifecycle;
manual policy runs use the same apply path and remain operator-triggered. A
policy run loads the configured catalog, plans compatible stopped and running
fleet upgrades, applies stopped nodes directly, rolls running nodes through
restart readiness plus supervised replacement, writes runtime
download/install/apply/restart events, and adds an aggregate policy-run audit
event that names ready and planned stopped/running rollout counts.

Catalog schema version 1 contains:

- `schema_version`: must be `1`.
- `generated_at_unix`: optional generation timestamp.
- `releases`: runtime release entries.
- `id`, `label`, `node_type`, `version`: release identity and runtime type.
- `platform.os`, `platform.arch`: Rust platform identifiers such as `macos`
  and `aarch64`.
- `url` or `download_url`: HTTPS package URL.
- `file_name` or `download_file_name`: safe local cache file name.
- `executable_name`: installed binary name.
- `expected_sha256`: package SHA-256.
- `max_bytes`: optional download limit; omitted entries use the default 512
  MiB limit.

## neo-cli Plugin Packages

The Plugin workspace can install a local ZIP package for the selected neo-cli
plugin while the node is stopped. The operator can hash the package in the
native app, paste or keep the expected SHA-256, and install the package into
`nodes/<id>/Plugins/<PluginId>/`.

Installation verifies the package SHA-256 before extraction, unpacks into a
staging directory under `Plugins/.neonexus/`, rejects absolute paths, parent
traversal, Windows drive-style paths, symbolic links, duplicate targets, and
attempts to write NeoNexus control data, then publishes the staged package by
replacing the previous plugin directory. Each install writes
`.neonexus/manifest.json`, persists installation metadata in SQLite, and adds a
`plugin-installed` runtime event.

## Private Network Launch Packs

The Role Planner can materialize one, four, or seven node private-network
templates for neo-cli, neo-go, or neo-rs, then export a native launch pack once
all planned nodes exist and are stopped. The launch pack writes each node's
managed runtime config, records each config's SHA-256 and each generated
artifact's SHA-256 in the schema v10 `manifest.json`, and adds a
`start-order.txt` file, `RUNBOOK.md`, `wallet-provisioning.json`,
`wallets/README.md`, `preflight-unix.sh`, `health-unix.sh`, `start-unix.sh`,
`stop-unix.sh`, `preflight-windows.ps1`, `health-windows.ps1`,
`start-windows.ps1`, and `stop-windows.ps1` under the workspace
`private-networks/` directory.

The exported runtime profile uses a deterministic private network magic per
runtime/template pair, consensus-node P2P endpoints as the seed list, the
planned consensus count as the validator count, and any operator-supplied
compressed committee public keys as the standby committee / validator roster.
Optional signer reference lines can attach a local wallet path, HTTP(S) signer
endpoint, and sidecar command template to each committee key using
`public_key | wallet path | signer endpoint | sidecar command template`. The
template supports `{wallet}`, `{endpoint}`, `{label}`, and `{public_key}`
placeholders; `{{` and `}}` escape literal braces. The launch manifest records
the template, the expanded command, and a structured `argv-no-shell` execution
plan for operator runbooks and audit.
The Roles workspace parses the same references before export and summarizes
required signer coverage, wallet gaps, endpoint references, sidecar commands,
generated no-shell argv plans, and the sidecar process specs that the native
supervisor can run later.
The Wallet Profiles workspace can append a selected profile as a signer
reference by reusing the wallet's validated contract public key and source
path; endpoint and sidecar command policy remain explicit operator inputs.
The verifier can also reload an exported `manifest.json` and rebuild those
sidecar process specs, so native supervision is not tied to the original
in-memory export session.
Generated preflight scripts check references when the wallet path is native to
the script platform. Unix/macOS preflight checks POSIX or relative wallet
paths, Windows preflight checks Windows or relative wallet paths, and
externally managed HTTP(S) signer endpoints are checked with
`NEONEXUS_SIGNER_TIMEOUT_SECONDS` as the timeout override. If a sidecar command
template is provided, generated start scripts parse it into binary plus args
and start that argv directly before node processes, without `sh -c` or
PowerShell `-Command`. Health scripts wait for its PID and optional endpoint,
and stop scripts terminate it after nodes are stopped. References and commands
are never injected as secret material. Consensus nodes are marked as
consensus-enabled in the generated neo-rs TOML.

The launch pack also writes `wallet-provisioning.json`, a structured checklist
that binds each committee public key to its referenced or recommended wallet
path, path scope, signer endpoint, and sidecar template. `wallets/README.md`
documents local wallet handling for disposable labs. Both files are hashed in
the launch manifest and validated offline. The validator also scans the
provisioning JSON before typed parsing so extra fields or command strings that
carry password, private-key, mnemonic, seed, token, or inline sensitive argv
markers fail the secret-boundary check even if artifact hashes were updated.
When a native-platform signer wallet path exists, the same validation pass also
loads that file and requires an encrypted Neo wallet with NEP-6 structure,
Base58Check account addresses, encrypted NEP-2 account keys, valid scrypt
parameters, account addresses that match their contract script hashes, a
contract public key that matches the manifest committee signer, and no plaintext
private-key/password/mnemonic/seed/token markers. Launch packs intentionally
contain no private keys, wallet passwords, or generated genesis key material;
actual encrypted Neo wallet creation remains an operator or external signer
step until a dedicated audited wallet authoring backend is integrated, while
validated metadata-only wallet profile import is available from the native
Wallet Profiles workspace and the headless CLI.
The generated preflight scripts check that node binaries, managed configs,
managed config SHA-256 values, work directories, signer references, and
required local ports are ready before startup, and detect live PID files from
an already-running pack. Native launch-pack validation performs the same
sidecar binary availability check for `argv-no-shell` plans: relative paths are
resolved from the pack root, bare binary names are resolved through `PATH`, and
foreign-platform paths are reported as warnings on the current host. Start scripts run preflight by default unless
`NEONEXUS_SKIP_PREFLIGHT=1` is set. They start nodes in deterministic plan
order, write per-node PID files and stdout/stderr logs into each managed work
directory, then run health probes unless `NEONEXUS_SKIP_HEALTH=1` is set.
Health probes wait for PID liveness, RPC/P2P/WebSocket ports, and JSON-RPC
`getblockcount` responses, using `NEONEXUS_HEALTH_TIMEOUT_SECONDS` and
`NEONEXUS_HEALTH_INTERVAL_SECONDS` as operator-tunable bounds. Stop scripts
stop nodes in reverse order for cleaner shutdown.

`neo-nexus --validate-launch-pack <manifest.json|launch-pack-dir>` performs an
offline Rust validation pass for exported packs. It checks schema version,
script files, generated artifact SHA-256 integrity, node binaries, managed
config files, managed config SHA-256 integrity, work directories, duplicate
ports, signer public keys, signer reference counts, native-platform signer
wallet files, encrypted NEP-6/NEP-2 wallet format, wallet provisioning
secret-boundary markers, signer wallet address/script-hash consistency, signer
wallet contract public key binding, and signer endpoint URL structure. A pack
with failed checks
prints the full report and exits with a non-zero status for CI or operator
handoff gates. The native Roles workspace
runs the same verifier immediately after export, shows the latest export
readiness summary and first blocking or warning check in the Private Network
panel, lets operators revalidate the same pack after fixing external wallet or
binary references, and records a runtime event with info, warning, or critical
severity. Application validation writes `validation-report.txt` and
`validation-report.json` into the launch pack directory; the CLI validation
command refreshes the same report files and prints their paths before exiting.
The handoff artifact therefore carries both human-readable and structured
readiness evidence whether it is checked from the desktop application or a
terminal. `RUNBOOK.md` summarizes platform commands, validation report files,
the secret material boundary, committee references, and node start order for
operators receiving the pack.

## Fast Sync Snapshot Catalogs

Fast Sync can import a local JSON catalog such as
`docs/snapshot-catalog.example.json`, or a signed HTTPS catalog with a detached
Ed25519 signature and trusted public key. HTTPS catalogs require a detached
signature before they are parsed. Local catalogs may also be signed when an
operator wants the same trust boundary for offline media.

Catalog entries are discovery metadata. Selecting an entry fills the normal
Fast Sync manifest draft, saving an entry creates the same manifest record as a
manual snapshot, and downloading an entry uses the existing HTTPS-only
redirect, size-limit, SHA-256, and verified-cache path. This keeps local
manifests, catalog manifests, and direct downloads under one validation model.

Import keeps the node stopped, re-checks the cached package SHA-256, and then
routes by snapshot package type. Raw snapshot files remain under
`data/<network>/fastsync/<snapshot-id>/` with a manifest. `.tar`, `.tar.gz`,
`.tgz`, and `.zip` packages are unpacked through a native Rust importer into
the node's managed `data/<network>/` runtime data root. Archive entries must be
relative regular files or directories; absolute paths, parent traversal,
Windows drive-style paths, symbolic links, NeoNexus control paths, expanded
size abuse, and existing-file collisions are rejected before publication.
For neo-rs, this aligns with `neo-node` TOML storage settings that use
`[storage] data_dir = "./data/<network>"`.

Snapshot catalog schema version 1 contains:

- `schema_version`: must be `1`.
- `generated_at_unix`: optional generation timestamp.
- `snapshots`: snapshot entries.
- `id` and `label`: snapshot identity shown in the native catalog picker.
- `network`: the target Neo network.
- `node_type`: compatible runtime, including `neo-cli`, `neo-go`, and
  `neo-rs`.
- `url` or `source_url`: HTTPS snapshot URL.
- `file_name` or `download_file_name`: safe local cache file name. Use `.acc`
  for raw packages, `.tar` / `.tar.gz` / `.tgz` for tar packages, or `.zip` for
  zip packages.
- `expected_sha256`: snapshot SHA-256.
- `max_bytes`: optional download limit; omitted entries use the default 64 GiB
  limit.
