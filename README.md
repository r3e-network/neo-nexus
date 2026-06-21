# NeoNexus

NeoNexus is a pure Rust native application for Neo N3 node operations. The
application is built with Rust, `eframe`/`egui`, and SQLite.

NeoNexus is not a WebView, browser shell, or frontend project. The first screen
is a desktop operations workbench: fixed inventory, workspace, inspector,
toolbar, and status areas that are designed for repeated node-management work
instead of document-style page scrolling.

The menu row, fixed toolbar, and keyboard accelerators share one native
command model. Operators can reload the workspace, create nodes, switch
workspaces, move through filtered inventory pages, and start, stop, or restart
the selected node without leaving the desktop workbench layout.

## Operator Workflow

1. Add or import node definitions for neo-cli, neo-go, or neo-rs.
2. Validate runtime binaries, generated configs, ports, and RPC health before
   launch.
3. Start, stop, restart, upgrade, and reconcile supervised node processes from
   native controls.
4. Use Operations to triage fleet readiness, selected-node readiness, port
   conflicts, event history, backup safety, support bundles, and integrity
   evidence.
5. Package and verify the native release artifacts from Settings or the
   headless CLI before handoff.

neo-rs is a first-class runtime target. NeoNexus recognizes the `neo-node`
binary, generates and validates TOML config files, supports RocksDB posture,
imports runtime and Fast Sync catalog entries, plans catalog upgrades, and
routes neo-rs readiness findings into the same native Operations workflow used
for neo-cli and neo-go.

## Requirements

- Rust 1.91+
- Platform GUI support for `eframe` on Linux, macOS, or Windows
- neo-cli, neo-go, or neo-rs `neo-node` binaries if you want to start real
  node processes

## Run

```bash
cargo run
```

Headless smoke checks for CI and packaging:

```bash
cargo run -- --self-check
target/release/neo-nexus --version
cargo run -- --runtime-smoke neo-rs /path/to/neo-node
cargo run -- --runtime-smoke-json neo-rs /path/to/neo-node
cargo run -- --rpc-health 10332
cargo run -- --rpc-health-json 10332
cargo run -- --workspace-readiness /path/to/neonexus.db
cargo run -- --workspace-readiness-json /path/to/neonexus.db
cargo run -- --workspace-metrics /path/to/neonexus.db
cargo run -- --workspace-metrics-json /path/to/neonexus.db
cargo run -- --workspace-metrics-prometheus /path/to/neonexus.db
cargo run -- --workspace-integrity /path/to/neonexus.db
cargo run -- --workspace-integrity-json /path/to/neonexus.db
cargo run -- --source-purity /path/to/neo-nexus
cargo run -- --source-purity-json /path/to/neo-nexus
cargo run -- --source-quality /path/to/neo-nexus/src
cargo run -- --source-quality-json /path/to/neo-nexus/src
cargo run -- --source-quality /path/to/neo-nexus/tests
cargo run -- --source-quality-json /path/to/neo-nexus/tests
cargo run -- --source-quality /path/to/neo-nexus
cargo run -- --source-quality-json /path/to/neo-nexus
cargo run -- --native-ui-audit /path/to/neo-nexus
cargo run -- --native-ui-audit-json /path/to/neo-nexus
cargo run -- --ci-policy /path/to/neo-nexus/.github/workflows/ci.yml
cargo run -- --ci-policy-json /path/to/neo-nexus/.github/workflows/ci.yml
cargo run -- --alert-preview datadog "https://event-management-intake.datadoghq.com/api/v2/events?api_key=<DD_API_KEY>" critical "RPC health unreachable"
cargo run -- --alert-preview-json datadog "https://event-management-intake.datadoghq.com/api/v2/events?api_key=<DD_API_KEY>" critical "RPC health unreachable"
cargo run -- --export-readiness-report /path/to/neonexus.db /path/to/reports
cargo run -- --export-event-journal /path/to/neonexus.db /path/to/events
cargo run -- --export-event-journal /path/to/neonexus.db /path/to/events 100 warning restart
cargo run -- --export-node-configs /path/to/neonexus.db /path/to/configs
cargo run -- --export-node-configs-json /path/to/neonexus.db /path/to/configs
cargo run -- --generate-node-config neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --generate-node-config-json neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --validate-node-config neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --validate-node-config-json neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --export-backup /path/to/neonexus.db /path/to/backups
cargo run -- --export-backup-json /path/to/neonexus.db /path/to/backups
cargo run -- --import-backup /path/to/neonexus.db /path/to/neonexus-backup.json
cargo run -- --import-backup-json /path/to/neonexus.db /path/to/neonexus-backup.json
cargo run -- --validate-backup /path/to/neonexus-backup.json
cargo run -- --validate-backup-json /path/to/neonexus-backup.json
cargo run -- --validate-wallet /path/to/validator.wallet.json
cargo run -- --validate-wallet-json /path/to/validator.wallet.json
cargo run -- --validate-launch-pack /path/to/private-network/manifest.json
cargo run -- --launch-pack-sidecars /path/to/private-network/manifest.json
cargo run -- --launch-pack-sidecars-json /path/to/private-network/manifest.json
target/release/neo-nexus --package-release dist
target/release/neo-nexus --verify-release-package dist
target/release/neo-nexus --verify-release-package-json dist
```

## Verify

```bash
cargo fmt --all --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test ci_policy
cargo test --test domain
cargo test --test repository
cargo run -- --self-check
cargo run -- --source-purity .
cargo run -- --source-quality src
cargo run -- --source-quality tests
cargo run -- --source-quality .
cargo run -- --source-quality-json .
cargo run -- --native-ui-audit .
cargo run -- --ci-policy .github/workflows/ci.yml
cargo run -- --alert-preview datadog "https://event-management-intake.datadoghq.com/api/v2/events?api_key=<DD_API_KEY>" critical "RPC health unreachable"
cargo run -- --workspace-metrics /path/to/neonexus.db
cargo run -- --workspace-metrics-json /path/to/neonexus.db
cargo run -- --workspace-metrics-prometheus /path/to/neonexus.db
cargo build --release
make verify
```

The native GUI binary is excluded from Cargo's default test harness and
`src/main.rs` uses an empty test-build entrypoint; tests run through the
library and named integration targets so filtered test commands and target
listing do not accidentally launch the desktop application.

## Current Native Feature Surface

- Fixed-panel desktop application layout.
- Native Operations remediation filters show facet-aware counts for both
  severity and target resolution workspace, so operators can keep a query or
  severity active while seeing how much work remains in Config, Logs, Monitor,
  Node Studio, Operations, Plugins, Roles, Runtimes, or Wallets before moving
  context.
- Native Settings release packaging controls that package the current
  executable, verify the ZIP/manifest/checksum trio, and record
  `release-packaged` / `release-package-verified` audit events.
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
  `--export-readiness-report`,
  `--export-support-bundle`, `--export-support-bundle-json`,
  `--export-event-journal`, `--export-node-configs`,
  `--export-node-configs-json`, `--generate-node-config`,
  `--generate-node-config-json`, `--validate-node-config`,
  `--validate-node-config-json`, `--export-backup`, `--export-backup-json`,
  `--import-backup`, `--import-backup-json`, `--validate-backup`,
  `--validate-backup-json`, `--validate-wallet`, `--validate-wallet-json`,
  `--import-wallet-profile`, `--validate-launch-pack`,
  `--launch-pack-sidecars`, `--launch-pack-sidecars-json`,
  `--package-release`, `--verify-release-package`, and
  `--verify-release-package-json` commands for packaging, operations, workspace
  readiness gates, machine-readable runtime, RPC, fleet, SQLite integrity,
  source purity, source quality, native UI shell, schema, alert route dry-runs,
  support bundles, event journal,
  generated node config, backup, and release reports, archived readiness
  evidence, integrity evidence, redacted diagnostics bundles, audit evidence,
  bulk config handoff evidence, standalone generated config evidence, backup
  export/import evidence, restore safety checks, encrypted Neo wallet validation
  with contract public key and address/script-hash evidence, metadata-only
  encrypted wallet profile import,
  launch-pack report refreshes, manifest-to-supervisor signer sidecar spec
  reports, native signer sidecar start/stop audit, release ZIP/checksum
  generation, release package verification, and CI smoke verification without
  opening a GUI window.
- Local SQLite workspace database.
- Node record creation, editing, deletion, searchable status-filtered Inventory
  listing, and a paged Overview fleet table that reuses the same filters while
  keeping row selection aligned with the active node.
- Node Studio runtime arguments are parsed and displayed through a shared
  no-shell argv formatter with quoted-value support, preserving paths with
  spaces while rejecting unterminated quotes.
- Runtime version metadata with `latest` or pinned version labels.
- Runtime Manager workspace for installing SHA-256 verified local runtime
  packages, downloading HTTPS runtime packages with HTTPS-only redirects,
  size limits, and atomic local cache publishing, optionally enforcing detached
  Ed25519 signatures, importing local and signed HTTPS runtime release
  catalogs, selecting the latest host-compatible release, filtering release and
  installed-runtime registries by runtime, host platform, trust state, and
  query, persisting runtime catalog source profiles, trusted signer profiles,
  and runtime inventory,
  planning fleet catalog upgrades, applying compatible runtimes to stopped
  nodes, and rolling selected or fleet running-node catalog upgrades through
  restart readiness plus supervised process replacement with stopped/running
  completion, skipped-node, blocked-attempt, and interrupted-run
  event-journal summaries that identify the failed node.
- Wallet Profiles workspace for importing encrypted NEP-6 Neo wallet metadata,
  usage/query-filtered saved profile registry, inspecting address, contract
  public keys, account counts, wallet SHA-256, validation/last-used timestamps,
  and applying the selected profile to Roles signer references, while recording
  `neo-wallet-profile-*` audit events without storing private keys, passwords,
  or copied wallet bytes.
- Runtime upgrade policy controls in Settings for scheduled catalog checks,
  signed-catalog enforcement, per-run fleet-node batch limits, UTC maintenance
  windows, rollout wave delays, manual policy runs, persisted last-check/apply
  metadata, ready/planned stopped/running rollout breakdowns, and event-journal
  audit records.
- Runtime binary preflight for configured node commands, including PATH
  resolution, host executable checks, neo-cli direct or `dotnet Neo.CLI.dll`
  wrapper recognition, neo-go binary recognition, and neo-rs `neo-node`
  recognition.
- Managed-config posture checks in fleet readiness and launch readiness,
  warning when neo-go or neo-rs runtime arguments already contain a config flag
  that will intentionally bypass NeoNexus generated managed config injection.
- Runtime smoke probes for configured node commands, using bounded
  `--version` / `--help` style checks with timeout handling and captured
  stdout/stderr summaries for neo-cli, neo-go, and neo-rs. Probe command
  evidence now includes the probed runtime path, bytes, and SHA-256 when
  available; stdout/stderr summaries and operator messages redact sensitive
  argv and output values before text or JSON export.
- JSON-RPC health probes for running nodes or explicit endpoints, checking
  `getversion` and `getblockcount`, classifying healthy/degraded/unreachable
  states, persisting selected-node RPC health records in SQLite, showing the
  latest health/height and recent H/D/U trend in native node panels, recording
  RPC health events, and retaining a bounded per-node health history.
- Automatic non-blocking RPC health monitoring for running nodes, with
  background probes, persisted Settings policy, configurable interval, Monitor
  visibility, status-change event recording, policy-change audit events, and
  per-node history pruning.
- Native process start/stop for configured node binaries, backed by a generic
  Rust managed-process supervisor that can also supervise non-node sidecars,
  with unified launch readiness rejecting critical binary, managed-config,
  lifecycle, active-node port, and IPv4/IPv6 localhost TCP listener blockers
  before launch, plus restart readiness that allows the node's own active
  listeners while still blocking conflicting active nodes. Supervisor log
  command evidence is redacted before it is written.
- Native port planning in Node Studio, with draft auto-assignment and
  selected-node repair that avoid existing node bindings and occupied localhost
  TCP ports on IPv4 or IPv6 loopback, preserving Neo RPC/P2P/WS port blocks
  and recording `node-ports-assigned` audit events for persisted repairs.
- Managed process stops request graceful termination first on Unix, wait through
  a bounded grace period, then force-kill only as a fallback; every stop writes
  structured stop evidence into the per-node log.
- Launch plan previews with runtime-specific effective arguments.
- Per-node stdout/stderr log capture for launched processes.
- Fixed-panel runtime log viewer with bounded tail reads and clear action.
- Runtime log search, match counts, latest jump, and tail-follow mode.
- Runtime log diagnosis for common startup failures such as occupied ports,
  config parse errors, permission problems, missing files, database locks, and
  runtime crashes, with actionable recommendations shown in the native Logs
  workspace and included in abnormal-exit notices.
- Native command menus, fixed toolbar actions, and keyboard accelerators for
  workspace reload, node creation, selected-node start, stop, lifecycle toggle,
  restart, primary workspace selection, and next/previous workspace cycling,
  plus filter-aware Alt/Option inventory navigation for previous/next node,
  page jumps, first node, and last node. Workspace, View, and Node menu entries
  dispatch through the same tested application commands as the New Node,
  Reload, Start, Stop, and Restart toolbar buttons.
- First-class selected-node lifecycle actions from the native toolbar, Node
  menu, Summary, and inspector surfaces, plus restart handling when a
  supervised child process has already exited.
- Runtime state reconciliation for exited children, stale previous sessions, and
  operator-confirmed missing PID records from the native Monitor workspace.
- Automatic watchdog restarts for abnormal exits with capped exponential backoff.
- Persisted watchdog policy configuration for enablement, retry attempts, base
  delay, and maximum delay from the native Settings workspace.
- Cross-platform system and node process resource telemetry with a fixed-panel
  Monitor workspace, including one-click refresh, missing-process repair, and
  a paged managed-process table with state, high-CPU, high-RSS, and query
  filters that keep node selection aligned with the visible process row.
  Telemetry health can focus missing running-node PIDs directly into the
  process table before repair.
- Structured runtime event journal for lifecycle, restart, config, logs, backup,
  plugin, and watchdog activity.
- Event journal search, severity filtering, match counts, selectable full
  event detail, node-event selection that synchronizes the active node, and
  bounded retention pruning from the Operations workspace, plus native filtered
  redacted text/JSON audit evidence export that records an
  `event-journal-exported` event. The same export is available headlessly from
  `--export-event-journal`, including optional limit, severity, and query
  filters.
- Native Operations and headless support bundle export with a redacted node
  inventory, readiness evidence, read-only SQLite integrity evidence, bounded
  event journal evidence, metrics text/JSON/Prometheus snapshots, redacted
  runtime log diagnosis summaries, SHA-256 file manifest, and ZIP archive.
  Support bundles are diagnostics evidence, not backups, intentionally exclude raw
  workspace databases, raw runtime logs, private keys, wallet passwords,
  passphrases, mnemonics, seeds, authorization bearer values, API keys,
  tokens, webhook secrets, cached runtime packages, and snapshots, and record
  `support-bundle-exported` audit events.
- Native Alerts workspace with disabled-by-default provider routing,
  Generic/Slack/Discord/Telegram/PagerDuty/Opsgenie/Datadog Events payload
  adapters, minimum severity thresholds, HTTPS or localhost-only endpoint
  validation, Telegram Bot API `chat_id` validation, PagerDuty Events API v2
  `routing_key` validation, Opsgenie Alert API v2 `api_key` validation with
  `GenieKey` delivery headers, Datadog Events intake v2 `api_key` validation
  with `DD-API-KEY` delivery headers, native and headless redacted route
  previews for operator and CI dry-runs, masked target entry, background delivery, local
  status/query-filtered delivery history, pruning, and policy-change audit events.
  Route secrets stay machine-local and are not included in JSON backups.
- Native Federation workspace for saved remote NeoNexus public endpoint
  profiles, URL normalization, status/query-filtered profile listing, manual
  and automatic read-only public status probes, fixed endpoint inspection,
  retained status/query-filtered paged probe history, enable/disable controls,
  configurable Federation monitor policy, pruning, and audit events for
  create/update/probe/delete/policy actions.
- Workspace backup schema v7 with remote federation profiles, plugin
  installation inventory, allowlisted
  workspace settings, runtime catalog profiles, trusted runtime signer
  profiles, metadata-only Neo wallet profiles, Fast Sync manifests, and recent
  runtime events.
- Node status and PID persistence.
- neo-cli, neo-go, and neo-rs node types.
- Mainnet, Testnet, and Private network selection.
- LevelDB and RocksDB storage engine settings.
- RPC, P2P, and WebSocket port settings with per-node non-zero and distinct
  port validation.
- Built-in plugin catalog with state/category/query-filtered native registry.
- Per-node plugin enablement state.
- neo-cli plugin ZIP package installation for stopped nodes, with SHA-256
  verification, native ZIP extraction, unsafe path rejection, manifest writing,
  SQLite installation inventory, and plugin-installed audit events.
- Role Planner workspace with RPC/API, State, Indexer, Consensus, and Observer
  presets for the selected node.
- neo-cli role application that updates plugin state and records an audit
  event, with neo-go and neo-rs role posture kept runtime/config managed.
- Private network topology planner for one, four, and seven node layouts across
  neo-cli, neo-go, and neo-rs runtimes.
- Private network template materialization that creates local private node
  definitions from an existing runtime template node, checks name/port
  conflicts, and applies role plugin state for neo-cli plans.
- Native signer handoff preview in the Roles workspace, parsing committee
  keys and signer references into signer, wallet, endpoint, sidecar, no-shell
  argv plan counts, and supervisor-ready sidecar process specs before export;
  exported launch pack manifests can be reloaded to reconstruct the same
  sidecar process specs for native supervision. The Roles workspace can load,
  start, stop, audit, watchdog-restart, and HTTP endpoint-check those signer
  sidecars through the Rust process supervisor. Sidecar execution is
  bundled-only by default; PATH or pack-external signer binaries require the
  persisted Roles `Allow External` policy and blocked starts plus policy
  changes are audited. Selected Wallet Profiles can add their contract public
  key and encrypted wallet path to the signer reference editor without copying
  wallet bytes or secrets.
- Private network launch pack export for stopped materialized plans, including
  deterministic network magic, validator count, seed list, operator-supplied
  committee public keys, signer wallet/HTTP(S) endpoint references, optional
  signer sidecar command templates with `{wallet}`, `{endpoint}`, `{label}`,
  and `{public_key}` expansion, no-shell argv execution plans for managed
  sidecars, per-node managed configs with SHA-256 inventory, schema v10 launch
  manifest, wallet provisioning checklist,
  no-secret wallet directory instructions, generated artifact SHA-256
  inventory, start order, handoff runbook, generated
  Unix/macOS shell plus Windows PowerShell preflight/start/health/stop scripts,
  startup guards for missing binaries, configs, config SHA-256 drift, work
  directories, live PID files, occupied local ports, native-platform signer
  wallet references, encrypted NEP-6/NEP-2 signer wallet validation with
  account address/script-hash consistency and committee public key binding,
  signer command validation, signer argv plan consistency, signer sidecar
  binary availability, signer sidecar PID files, and signer
  endpoint reachability, plus post-start signer, node PID, port, and
  JSON-RPC health probes. Launch packs are
  automatically validated after native app export, can be validated again from
  the native Roles workspace after operator fixes, can be validated offline
  from the native CLI before handoff, verify managed config and generated
  artifact integrity, verify manifest sidecars can be recovered as native
  supervisor process specs, start, stop, watchdog-restart, endpoint-check, and
  persisted bundled-only execution-policy gate recovered signer sidecars from
  the native Roles workspace, expose those specs as text or JSON through
  `--launch-pack-sidecars`, refresh text and JSON validation reports inside the
  launch pack directory, reject signer wallet files that are missing, unencrypted,
  not NEP-6/NEP-2, have account addresses that do not match their contract
  scripts, do not expose a matching contract public key, or carry plaintext
  private key material, reject wallet provisioning JSON that carries password,
  private-key, mnemonic, seed, token, or inline sensitive argv markers, and
  explicitly exclude private keys and passwords.
- Fast Sync workspace for registering local or HTTPS snapshot manifests,
  importing local or signed HTTPS snapshot catalogs, selecting catalog
  snapshots, filtering catalog and registry entries by runtime, network,
  verification, cache state, and query, downloading HTTPS snapshots with
  HTTPS-only redirects and size limits, validating SHA-256, caching verified
  snapshot files into the native workspace, importing raw `.acc` files or
  unpacking `.tar`, `.tar.gz`, and `.zip` snapshot packages into stopped nodes'
  managed data directories, and rejecting unsafe archive paths or import target
  conflicts before publish.
- Operations readiness diagnostics for runtime binary preflight, generated
  config validation, versions, lifecycle state, configured and live localhost
  IPv4/IPv6 port conflicts, and plugin alignment, with Node Studio port repair
  available for stopped definitions and the same fleet gate available from
  `--workspace-readiness <neonexus.db>` and
  `--workspace-readiness-json <neonexus.db>`. Operators can also export
  timestamped text and JSON evidence from the native Operations workspace or
  `--export-readiness-report <neonexus.db> <output-dir>`. The native action
  queue flattens fleet findings into a paged, severity/query-filtered operator
  list that prioritizes blocking nodes, can focus critical or warning work in
  one click, opens the matching native resolution workspace for the selected
  finding, and keeps row selection aligned with the active node. The port
  matrix is also paged, can focus blocked port rows in one click, and can be
  filtered by node status, chain network, port health, or query while keeping
  the selected node synchronized with the visible row. Selected-node readiness
  checks expose native severity/query filtering, one-click critical/warning
  focus, selectable detail, resolution workspace shortcuts, and pagination for
  focused triage. Action queue and selected-node readiness queries can match
  resolution keys, workspace labels, action labels, and operator hints, and
  both panels can filter directly by target resolution workspace. Workspace
  filter menus and severity buttons show facet-aware counts so operators can
  see how much matching work belongs to each resolution area or severity
  bucket before switching context.
  Headless workspace readiness text/JSON and exported readiness reports include
  stable resolution keys, workspace labels, action labels, and operator hints
  for automation handoff, so GUI triage, CI gates, and support bundles all
  describe the same next native workspace instead of relying on prose-only
  remediation.
- Readiness resolution workflow is first-class native application behavior:
  each diagnostic carries a stable resolution identity, the action queue and
  selected-node readiness panel preserve severity/query context while filtering
  by target workspace, selected rows remain tied to the active node, and the
  resolution button moves the operator directly to Config, Logs, Monitor, Node
  Studio, Operations, Plugins, Roles, Runtimes, or Wallets. This keeps triage
  in fixed panels instead of forcing operators through a scrolling document or
  prose-only checklist.
- Headless workspace metrics from `--workspace-metrics <neonexus.db>` and
  `--workspace-metrics-json <neonexus.db>` capture system CPU/memory pressure,
  managed node process CPU/memory/uptime, and stale running-node PID
  detection for CI jobs, operator scripts, and external monitoring collectors.
  `--workspace-metrics-prometheus <neonexus.db>` emits the same snapshot in
  Prometheus text exposition format for scrape pipelines and metrics agents.
- Read-only workspace integrity checks from the native Operations workspace or
  through `--workspace-integrity` and `--workspace-integrity-json`, covering
  SQLite `integrity_check`, foreign-key violations, required tables, required
  columns, required indexes, row counts, and database page metadata without
  mutating the workspace. The desktop action records a dedicated
  `workspace-integrity-checked` audit event.
- Workspace backup export/import for node definitions, plugin state, plugin
  installation inventory, remote federation profiles, allowlisted application
  settings, runtime catalog profiles, trusted signer profiles, Fast Sync
  manifests, and recent events, with restored nodes safely reset to stopped.
  Operators can run the same
  export path headlessly against an existing workspace database with
  `--export-backup <neonexus.db> <output-dir>` or
  `--export-backup-json <neonexus.db> <output-dir>` for scheduled jobs and CI
  evidence. Disaster recovery and migration scripts can restore with
  `--import-backup <neonexus.db> <backup.json>` or
  `--import-backup-json <neonexus.db> <backup.json>` after the backup passes
  validation, and the CLI refuses to import into workspaces with active
  running/starting nodes. The native Operations workspace can validate the
  latest backup before import, requires that current validation before enabling
  native import, keeps the last validation counts visible in Workspace Safety,
  and records a `backup-validated` audit event. The same deep validation is
  available from
  `--validate-backup <backup.json>` and
  `--validate-backup-json <backup.json>`, including duplicate identifier,
  duplicate plugin inventory, and duplicate node-port rejection. Runtime event
  history messages are redacted during export and restored idempotently so
  repeated imports do not duplicate identical audit records. Local runtime
  installations and cached snapshot files and remote probe telemetry remain
  machine-local and are not copied by JSON backup.
- Dashboard summary projections.
- neo-cli JSON, neo-go YAML, and neo-rs TOML configuration previews, local
  exports, and generated-config validation with runtime-specific parse and
  semantic checks before managed writes.
- Headless bulk node config export that writes all neo-cli JSON, neo-go YAML,
  and neo-rs TOML configs under collision-safe per-node directories and emits
  timestamped text/JSON evidence for CI, handoff, and audits.
- Headless standalone node config generation for neo-cli JSON, neo-go YAML,
  and neo-rs TOML without a workspace database, writing the target file,
  revalidating it immediately, and emitting text/JSON generation evidence.
- Headless node config validation for neo-cli JSON, neo-go YAML, and neo-rs
  TOML files, with text/JSON evidence and non-zero exit codes for semantic
  drift such as network, storage, or port mismatches.
- Runtime-specific managed configuration application with audit events:
  neo-cli publishes `config.json` into the node workdir, while neo-go and
  neo-rs use managed files under `nodes/<id>/config/`; running nodes can use
  Apply + Restart for a controlled config rollout through the native
  supervisor.
- neo-go launch planning that injects the managed `node --config-file
  <node.yml>` command when needed, and neo-rs launch planning that injects the
  managed `--config <node.toml>` when needed; existing runtime config flags
  are preserved for advanced operators and surfaced as readiness review
  findings instead of being silently overridden.
- Per-node working directories so relative runtime data paths stay inside the
  native workspace.

## Architecture

```text
src/
  app.rs                  application state, actions, and eframe lifecycle
  argv.rs                 argv text parsing and quote-safe command display
  redaction.rs            shared diagnostics/event secret redaction
  app/
    draft.rs              node creation draft conversion
    paging.rs             fixed-panel pagination helpers
    text.rs               text clipping helpers
    theme.rs              native UI color and status styling
    view.rs               top-level workbench tab model
    views.rs              workbench dispatcher and tab strip
    views/
      shell.rs            menu bar, toolbar, inventory, inspector, status bar
      overview.rs         operations summary and fleet snapshot
      operations.rs       readiness checks, action queue, port matrix
      alerts.rs           provider alert routing and delivery history
      monitor.rs          system and managed-process telemetry panels
      settings.rs         native runtime policy and workspace path controls
      runtimes.rs         runtime download, install, and apply workflow
      federation.rs       remote public endpoint profile workspace
      wallets.rs          encrypted Neo wallet profile workspace
      nodes.rs            node definition studio
      roles.rs            role presets and private network topology planner
      snapshots.rs        fast sync manifest, catalog, import, and cache controls
      plugins.rs          plugin catalog, package installation, and details
      config.rs           paged native config preview and export controls
      logs.rs             paged process log viewer
      operations/         split action queue, port matrix, event journal, readiness, and safety panels
    widgets.rs            reusable native UI primitives
  app/operations_flow/    non-visual Operations actions, resolution routing, filters, reports, backup validation, and retention
  backup.rs               workspace backup export/import
  catalog.rs              plugin definitions
  plugins.rs              neo-cli plugin ZIP installation and manifest writing
  preflight.rs            runtime binary resolution, identity, and launch checks
  rpc_health.rs           Neo JSON-RPC getversion/getblockcount health probes
  runtime_smoke.rs        bounded runtime binary smoke probes
  config.rs               neo-cli JSON, neo-go YAML, neo-rs TOML validation/export/reporting
  dashboard.rs            summary projections
  diagnostics.rs          fleet and node readiness checks with resolution metadata
  readiness_report.rs     timestamped text/JSON readiness evidence and resolution exports
  support_bundle.rs       redacted diagnostics support bundle export
  source_purity.rs        pure Rust source tree boundary gate
  source_quality.rs       Rust marker, module, docs, and CI file budget gate
  workspace_integrity.rs  read-only SQLite/schema/foreign-key integrity checks
  federation.rs           remote endpoint profile model and public status probe
  alerts.rs               alert routing policy, provider payloads, delivery status
  event_journal_report.rs timestamped text/JSON runtime event audit exports
  events.rs               runtime event journal model
  launch.rs               effective launch command planning
  logs.rs                 bounded log tailing, search, and failure diagnosis
  metrics.rs              system and managed-process resource snapshots
  port_planner.rs         local TCP-aware node port block assignment
  private_network.rs      private network launch pack export
  wallet.rs               encrypted Neo wallet validation, profile import, address/script checks, secret-boundary checks
  repository.rs           SQLite schema and persistence APIs
  roles.rs                runtime role presets and private network plans
  runtime.rs              release catalog import, HTTPS download, verification, install plans
  app/runtime_upgrade.rs  native runtime install, catalog upgrade, and policy actions
  app/health_events.rs    RPC/remote health event severity and dedupe helpers
  app/sidecar_health.rs   signer sidecar execution policy and endpoint health helpers
  app/managed_config_flow.rs managed config, log, and workspace path actions
  app/private_network_flow.rs private-network role, launch-pack, and sidecar actions
  app/workflow.rs         non-visual application workflow drafts and notices
  app/tests.rs            app-level behavior tests kept out of the main application module
  runtime_smoke.rs        bounded runtime binary smoke probes
  snapshots.rs            fast sync snapshot hashing, archive import, and local cache workflow
  supervisor.rs           child process lifecycle and log redirection
  types.rs                shared domain model
  watchdog.rs             bounded automatic restart policy
tests/
  domain.rs                         shared integration-test fixtures and module map
  domain/
    backup_diagnostics.rs           backup, readiness, and diagnostics behavior
    config_launch_supervisor.rs     config generation, launch planning, and process supervision
    plugins_snapshots.rs            plugin package and fast-sync snapshot workflows
    roles_private_network.rs        runtime roles, metrics, and private-network launch packs
    runtime_federation.rs           runtime catalog, package, upgrade, and federation workflows
  repository.rs
```

Runtime data is stored in the platform data directory by default, or in
`NEONEXUS_DATA_DIR` when that environment variable is set.

## Documentation

The documentation set is part of the native application contract. It records
what the desktop app does today, what the validation gates prove, and which
sample catalogs operators can import without changing the Rust-only boundary.

- [Native Rust App](docs/native-rust.md)
- [Operator Benchmarks](docs/operator-benchmarks.md)
- [Validation Report](docs/native-validation.md)
- [Runtime Catalog Example](docs/runtime-catalog.example.json) for
  Linux/macOS/Windows runtime release metadata.
- [Snapshot Catalog Example](docs/snapshot-catalog.example.json) for signed
  Fast Sync snapshot catalogs.

## Remaining Work

NeoNexus is now structurally pure Rust, but the native implementation still has
product gaps to fill before it can replace the old production feature surface:

- `--source-purity` and `--source-purity-json` now make the pure Rust boundary
  executable by rejecting Node/Web manifests, frontend source files,
  `node_modules`, web/frontend directories, Docker/compose artifacts, and
  nginx web-server deployment files in the current source tree, plus
  WebView/Tauri project files, Cargo dependencies, and lockfile packages.
- `--source-quality` and `--source-quality-json` make the production
  panic-oriented marker, document-style native layout container, and
  hardcoded platform shortcut label, and oversized Rust source file rules
  executable for local and CI verification;
  Rust modules must stay within the 200-line professional module budget, and
  test sources may use assertion shortcuts and exact shortcut-label
  expectations while still staying under that same source-size budget.
  Repository-root scans also reject JSON, Markdown, TOML, YAML, and named
  maintenance files such as `Makefile`, `LICENSE`, and `NOTICE` over 1000
  lines with case-insensitive matching, so catalog examples, README, docs,
  Cargo metadata, and CI workflow files stay reviewable. Text and JSON
  evidence include Rust and maintenance-file scan counts, source snippets, and
  remediation hints for CI review, making each failure actionable without
  opening the source tree by hand.
- `--native-ui-audit` and `--native-ui-audit-json` make the positive native
  application shell contract executable by requiring the eframe/egui desktop
  entry point, fixed header/status/inventory/inspector/workspace panels,
  minimum window sizing, and explicit fixed workspace tabs while rejecting
  scroll/WebView/Tauri/Wry UI markers.
- `--ci-policy` and `--ci-policy-json` make the cross-platform native release
  policy executable by requiring Linux, macOS, and Windows CI coverage,
  release packaging checks, neo-rs runtime/config gates, runtime-smoke
  passed/hash evidence checks, source purity, source quality, native UI audit,
  and no frontend/Node/WebView CI tooling.
- Runtime-specific in-process live reload without restart where runtimes
  support it.
- Audited encrypted Neo wallet generation and richer wallet-authoring flows for
  private networks; the native application now validates, imports, displays,
  audits, backs up, and restores metadata-only wallet profiles, while launch
  packs validate operator-provided encrypted NEP-6 signer wallets against
  committee public keys and include first-class no-secret provisioning evidence.
- Additional provider integrations beyond the current alert routes and
  headless metrics exports, such as logging, uptime, and broader
  incident-management services.
- Linux and Windows smoke tests against real neo-cli / neo-go / neo-rs
  binaries.
