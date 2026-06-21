# Operator Benchmarks

This document records the product patterns used to guide the native Rust
NeoNexus workbench.

## References

- Dappnode focuses package management around operator panels such as Info,
  Config, Network, Logs, File Manager, and Backup:
  <https://docs.dappnode.io/docs/user/packages/understanding-dappnode-packages/overview/>
- Stereum emphasizes guided setup, a dashboard, monitoring tools, and flexible
  node configuration: <https://stereum-dev.github.io/ethereum-node-web-docs/>
- Umbrel's Bitcoin Node highlights explicit version choice, advanced network
  settings, and clear compatibility guidance:
  <https://apps.umbrel.com/app/bitcoin>
- Rocket Pool Smartnode documentation treats node operation as an ongoing
  lifecycle: secure, maintain, monitor, and upgrade:
  <https://docs.rocketpool.net/node-staking/cli-intro>
- ethereum.org describes mature launchers as tools that automate setup and
  provide guided setup, monitoring, and control:
  <https://ethereum.org/developers/docs/nodes-and-clients/run-a-node/>
- neo-rs is the Rust Neo N3 node implementation that ships a `neo-node`
  daemon and TOML config files:
  <https://github.com/r3e-network/neo-rs>

## Applied Patterns

- Workbench over pages: a fixed native shell with Inventory, Workspace,
  Inspector, and Status areas.
- Native command discoverability: common operator actions live in stable
  desktop menus, a compact toolbar, and keyboard accelerators backed by the
  same command dispatcher, so reload, node creation, view switching, inventory
  movement, and selected-node lifecycle controls are reachable without
  changing layout context.
- Operator-first triage: mature node tools guide operators from a warning to a
  concrete maintenance surface. NeoNexus applies the same pattern by attaching
  every readiness finding to a native resolution workspace, action label, and
  operator hint, then showing facet-aware workload counts before the operator
  switches severity or target workspace.
- Source purity: `--source-purity` and `--source-purity-json` reject Node/Web
  manifests, frontend source files, `node_modules`, web/frontend directories,
  Docker/compose artifacts, nginx deployment files, WebView/Tauri project
  files, and WebView/Tauri Cargo packages so the repository remains a native
  Rust application rather than a browser, webview, or server-container project
  in disguise.
- Headless observability: `--workspace-metrics` and
  `--workspace-metrics-json` expose system pressure, managed-node resource use,
  and missing running-node PIDs for automation and external monitoring.
  `--workspace-metrics-prometheus` emits the same data as Prometheus text
  exposition for scrape-based monitoring.
- Explicit operator state: version intent, lifecycle status, PID, plugin state,
  installed runtime inventory, exported config, managed config, and network
  ports are visible in the application.
- Runtime provenance: local runtime packages are accepted only after SHA-256
  verification and current-platform matching; HTTPS runtime downloads allow
  only HTTPS redirects, enforce size limits, and stream into an atomic local
  cache; detached Ed25519 signatures can be enforced before packages are
  installed under the managed workspace with a manifest; trusted signer
  profiles keep reusable Ed25519 public keys separate from catalog source
  profiles.
- Runtime-specific configuration: NeoNexus keeps JSON exports for neo-cli,
  YAML exports for neo-go, and TOML exports for neo-rs `neo-node`, with
  headless bulk export reports for CI, reviews, and operator handoff.
- neo-rs parity: the Rust node is treated as a production runtime, not a demo
  path. Runtime preflight recognizes `neo-node`, config generation and
  validation use TOML, storage posture favors RocksDB, Fast Sync and runtime
  catalogs can carry neo-rs entries, and Operations routes neo-rs findings into
  the same remediation workflow as other Neo runtimes.
- Launch parity: command previews and process starts share the same launch
  planner, so managed config paths are visible before an operator starts a
  node; explicit neo-go or neo-rs config flags are preserved for advanced use
  but shown as readiness review findings because they bypass generated managed
  config injection.
- Workspace integrity: operators can run read-only SQLite integrity, schema,
  index, row-count, and foreign-key checks as text or JSON before backups,
  migrations, or release handoff.
- Log evidence: node stdout/stderr is redirected into stable local files so
  operators can inspect runtime output after a start attempt or long-running
  session.
- Native log inspection: captured output is searchable and shown in a fixed,
  paginated workspace with tail-follow controls instead of a scrolling web
  document.
- Native shell audit: CI can prove the desktop surface uses eframe/egui fixed
  panels with explicit workspaces and no WebView/Tauri/Wry or document-style
  scrolling markers.
- Source-size discipline: CI source quality gates keep production and test
  Rust files under the module line budget and keep catalog JSON, documentation,
  Cargo, named maintenance files, and CI maintenance files under a 1000-line
  review budget so operational domains continue to move into focused native
  modules instead of accumulating in monolithic files.
- Platform-native shortcut labels: production code cannot hardcode `Cmd+`,
  `Ctrl+`, `Option+`, or `Alt+` menu strings; labels are generated through the
  native command formatter so macOS and Linux/Windows operators see familiar
  shortcut names while tests can still assert exact expected text.
- Actionable quality evidence: source-quality failures include the source
  location, blocked marker, snippet, and remediation hint, matching the
  operator-product principle that a warning should lead directly to the next
  concrete maintenance action instead of leaving maintainers to infer intent.
- Runtime reconciliation: supervised process exits update persisted node state,
  and stale runtime records from previous application sessions are cleared on
  startup.
- Watchdog policy: abnormal exits are retried with capped exponential backoff,
  stop after a bounded number of attempts, and can be tuned from native
  Settings without editing files.
- Resource monitoring: host CPU/RAM pressure and managed node process CPU,
  RSS, uptime, and missing-PID state are visible without leaving the app, and
  Telemetry health can focus missing PIDs into the managed-process table before
  an operator repairs stale runtime records.
- Event journal: lifecycle, configuration, plugin, backup, log, and watchdog
  activity is persisted as structured operator history, shown in Operations,
  selectable for full fixed-panel detail with node-event selection synchronized
  to the active node, and exportable from Operations or headlessly as filtered
  timestamped text/JSON audit evidence.
- Support handoff: native Operations and headless diagnostics bundles combine
  readiness, read-only integrity, metrics text/JSON/Prometheus snapshots,
  bounded event journal, redacted node inventory, privacy notes, SHA-256
  manifests, ZIP archives, and audit events so operators can share evidence
  without handing over raw databases, wallet secrets, webhook endpoints,
  runtime packages, or snapshot caches.
- Journal hygiene: event history can be filtered by severity or text, counted
  against the full journal, selected for full event inspection, and pruned to
  retain the newest operational history.
- Federation hygiene: saved remote endpoint profiles and retained probe
  history are both status/query filtered and paged so multi-site operators can
  inspect remote health without a scrolling document surface.
- Backup safety: workspace backups can be exported from the native UI or
  headlessly with text/JSON CLI evidence, imported locally or headlessly with
  text/JSON restore evidence, and validated before restore from Operations or
  headless CLI, with native import requiring current validation evidence in
  Workspace Safety, including plugin state and plugin installation inventory,
  while imports are blocked for workspaces with active nodes and restored node
  definitions are kept stopped rather than reviving stale PIDs.
- Release handoff: Settings can package the current native executable, verify
  the ZIP/manifest/checksum trio, and record release audit events; the same
  release flow remains available headlessly for CI.
- Readiness checks: NeoNexus evaluates binary availability, version posture,
  lifecycle consistency, port conflicts, and plugin alignment before operators
  treat a node as production-ready, with selected-node critical/warning focus
  and selectable check detail in Operations. Native and headless readiness
  evidence carries a stable resolution key plus the recommended
  workspace/action/hint for resolving each finding.
- Resolution handoff: every readiness finding points to a concrete native
  workspace, matching mature node-management tools that route operators from
  a warning directly into config, logs, runtime, plugin, monitor, role, wallet,
  or node-editing work from either the fleet action queue or the selected-node
  check detail instead of leaving remediation as free-form text. The same
  handoff metadata is searchable by stable key, label, action, or hint so
  operator terms and automation terms converge, and workspace filters let an
  operator narrow the visible work to a target remediation area while keeping
  severity and query context intact.
- Native triage continuity: the active node, selected readiness row, severity
  facet, query facet, and target workspace facet are treated as persistent
  operator context. This mirrors professional node tools where an operator can
  drill into one remediation area, jump to the owning workspace, and return to
  the same bounded work queue without losing the investigation thread.
- Workload visibility: remediation workspace filters show counts under the
  active severity/query context, and severity buttons show counts under the
  active query/workspace context, giving operators dashboard-like cues before
  they switch from one bounded task area to another.
- Port matrix: conflicting RPC/P2P/WS bindings can be focused directly from
  Operations, keeping the selected node aligned with the visible blocked row.
- Action queue: critical and warning diagnostics are surfaced as operational
  work, can be focused by severity in one click, and keep selected actions
  synchronized with the active node while opening the right native workspace
  for resolution instead of hiding inside configuration views.
- Operations module discipline: action queue, readiness checks, port matrix,
  event journal, metrics, and workspace safety are implemented as focused
  native panels with separate non-visual workflow modules, keeping the desktop
  application maintainable as the operator surface grows.
- Role presets: common node duties are represented as explicit operator
  postures, with plugin changes separated from runtime-managed configuration.
- Plugin package provenance: neo-cli plugin ZIP packages are accepted only for
  stopped nodes after SHA-256 verification, extracted through native Rust
  staging, recorded with a local manifest, and persisted in SQLite inventory.
- Private topology planning: multi-node layouts are previewed with deterministic
  roles and ports, then materialized as local node definitions only after
  source runtime and name/port conflicts are checked.
- Private launch packs: materialized private-network layouts export managed
  configs, deterministic network magic, consensus seed lists, validator count,
  operator-supplied committee public keys, launch manifest, start order, and
  cross-platform preflight/start/health/stop checks for signer wallet
  references, HTTP(S) signer endpoints, and optional signer sidecar command
  templates with no-shell argv execution plans, without silently inventing
  signer keys. The Roles workspace previews signer, wallet, endpoint, sidecar,
  no-shell argv plan, and supervisor-ready sidecar process coverage before
  export, and exported manifests can be reloaded into the same native process
  specs; `--launch-pack-sidecars` and `--launch-pack-sidecars-json` expose
  those specs for operator review and automation. Each pack includes a
  runbook for platform commands, report files, committee references, expanded
  signer commands, `wallet-provisioning.json`, `wallets/README.md`, and
  secret-boundary reminders. Native CLI validation can audit an exported pack
  before handoff, fail if wallet provisioning JSON carries password,
  private-key, mnemonic, seed, token, or inline sensitive argv markers, and
  fail CI when required binaries, configs, work directories, scripts, signer
  wallets, encrypted NEP-6/NEP-2 signer wallet structure, signer wallet contract
  public key binding, signer wallet address/script-hash consistency, or signer
  sidecar binaries are missing or when manifest invariants are missing;
  the application also runs the same verifier after export, surfaces the first
  issue, supports same-pack revalidation after
  fixes, writes text and JSON validation reports into the pack, and records the
  readiness outcome in the event journal.
- Fast Sync cache discipline: snapshot manifests and signed catalogs carry
  explicit SHA-256 values, local sources are hashed before trust, remote
  catalogs require detached Ed25519 signatures, and cached files are published
  only after the digest matches.
- Snapshot import discipline: archive snapshots are unpacked by native Rust
  code from a verified cache, use staging before publication, reject unsafe
  paths and symbolic links, and fail on existing target files instead of
  overwriting chain data silently.
- Data directory discipline: launched nodes run from per-node working
  directories, and Fast Sync snapshots are staged only into stopped nodes'
  managed data directories.
- Config discipline: generated exports are separated from per-node managed
  config files under `nodes/<id>/config/`, and application events record when a
  config is staged for restart/reload.
- Paginated fixed panels: long lists are paginated to preserve desktop
  application layout and avoid document-style scrolling.
- Native application contract: source quality and native UI audit gates keep
  the operator surface in a fixed-panel Rust desktop form instead of drifting
  back toward a browser page, WebView shell, or scroll-first layout.

## Current Boundaries

NeoNexus now models version intent, exposes common commands through native
menus, toolbar buttons, and keyboard accelerators, exports generated config
files, captures and displays process logs, reconciles process state, applies
bounded watchdog restarts, exposes persisted watchdog policy controls, records
structured runtime events with selectable native audit detail, exposes resource
telemetry, exports/imports workspace backups, applies role presets where plugin
state is the right abstraction, previews private-network topologies, and
materializes those templates into local node definitions. It also exports
private-network launch packs with managed
configs, deterministic seed lists, validator counts, operator-supplied
committee public keys, manifests, and start orders for stopped materialized
layouts. It also registers local or HTTPS Fast Sync manifests, downloads
HTTPS snapshots through HTTPS-only redirects into a bounded cache, publishes
only SHA-256 matching snapshot files, imports local or signed HTTPS Fast Sync
snapshot catalogs, converts catalog entries into normal manifests, imports raw
snapshot files or unpacks `.tar`, `.tar.gz`, and `.zip` packages into stopped
nodes' managed data directories, and applies generated configuration into
per-node managed config paths. It also downloads HTTPS runtime packages through HTTPS-only redirects
into a bounded local cache, installs SHA-256 verified local runtime packages,
enforces detached Ed25519 signatures when a signature and trusted public key
are provided, persists the runtime inventory, and applies compatible installed
runtimes to stopped node definitions. It also imports local runtime release
catalogs and signed HTTPS runtime release catalogs to populate verified package
download/install drafts, and persists reusable catalog source profiles with
last-load metadata. It also persists reusable trusted signer profiles with
valid Ed25519 public keys and last-used metadata, then applies those trust
anchors explicitly to catalog or package verification. For selected nodes and
fleet candidates it can plan the latest compatible catalog release and run a
managed download, install, event-recording, and apply flow; stopped nodes are
updated directly, while running nodes pass through restart readiness and native
supervisor process replacement, and completion notices distinguish stopped
direct-apply work, running restart-rollout work, blocked active nodes, and
current/unavailable nodes while event-journal summaries keep the batch
searchable when a run applies, interrupts, no-ops, or is blocked before
planning, with interrupted runs identifying the failed node. It also persists a
scheduled runtime upgrade policy that binds to a
saved catalog profile, requires signed catalogs by default, limits each run to
fleet-node batches, supports UTC maintenance windows and delayed rollout waves,
records last-check/apply metadata, and writes aggregate audit events that
distinguish ready and planned stopped direct-apply work from running
restart-rollout work. It also includes generic,
Slack, Discord, Telegram, PagerDuty, and Opsgenie alert routing with local
delivery history,
metadata-only Neo wallet profile import/registry/audit and Roles
signer-reference application, plus private-network launch-pack signer sidecar
command-template orchestration and native Roles start/stop/audit/watchdog
restart plus HTTP endpoint health checks for recovered sidecar process specs,
with bundled-only execution by default, an explicit persisted Roles override
for external/PATH signer binaries, workspace backup of that policy, and
audited policy changes plus blocked starts. Remote Federation profiles retain
status/query-filtered paged probe history for endpoint inspection. The process
supervisor is now generic enough for node daemons and
signer sidecars, but it does not yet support runtime-specific live config
reloads for already-running nodes, deeper signer protocol checks beyond HTTP
reachability and process execution policy, audited encrypted private-network
wallet generation and richer authoring UI, or additional provider integrations
beyond the current alert routes and headless metrics exports, such as logging,
uptime, and broader incident-management services. Those remain future native Rust
features.
