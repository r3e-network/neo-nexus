# Changelog

All notable changes to NeoNexus are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Erratum for 1c4114c

Commit 1c4114c ("mount the integration suite, and stop calling plaintext
encrypted") overstated what it did, and it left CI red.

- It said 68 previously invisible tests were now running. Of those 68, 18 made
  real assertions about the product. The other 50 asserted nothing about it:
  they ran against a test-local `NodeManager` with a fixed PID 12345 (the real
  `neo_nexus::node_manager::NodeManager` never existed), exercised fixtures,
  helpers and a mock metrics server, only printed "coverage", or used asserts
  such as `result.is_ok() || result.is_err()` that cannot fail. Those 50 are
  gone. The 18 real ones are kept in `tests/integration/node_types.rs` or moved
  to library unit tests.
- It said the repository has no crypto dependency. `Cargo.toml` lists aes,
  scrypt, p256, ed25519-dalek and sha2.
- It failed CI's `Format` step and had 6 Clippy errors, all in code it added.
- The binary's own `--help` still called the backup encrypted.

`cargo test --test integration` now runs 17 tests, all through the public
`neo_nexus` API with no test-local stand-ins for product code. Nine drive the
lifecycle core shared by the web workbench and
`--node-start`/`--node-stop`/`--node-restart` (`execute_node_launch`,
`stop_node_runtime`) over a real SQLite workspace, with the real
`LaunchPlanner`, `ConfigExporter` and `ProcessSupervisor` spawning a real child
process. For each of neo-cli, neo-go, neo-rs, neox-geth and neox-rs, a start
writes a parseable managed config at the path the launch command names, records
`Running` with the pid of a live process and logs the launch; a stop ends that
process, records `Stopped` with no pid and logs the stop. Restart replaces the
process. All five run side by side under one supervisor while a node with a
missing runtime fails alone in `Error`. The other eight check node-id and port
validation, node-type round-trips, chain family, default storage engine,
plugin support and config-path extensions. The runtime is a compiled stand-in
that ignores its arguments and sleeps, so no real Neo client, sync, RPC,
readiness check, signer duty or plugin is exercised.


### Security

- **Backup is described as what it is everywhere**: `--help` now reads "Write a
  0600 plaintext JSON workspace export, NOT encrypted", and the documented-claims
  gate reads the rendered `--help` and whole statements rather than single lines,
  so reworded, wrapped or ambiguous encryption claims about the backup fail it.
  `docs/MIGRATION-v4.3.md` no longer tells operators to run a
  `make backup-export FORMAT=tar.gz` target that does not exist.
- **Dependency advisory cleanup**: upgrade `rustls` to 0.23.45 (RUSTSEC-2026-0285,
  TLS 1.3 handshake message boundary) and `anyhow` to 1.0.104 (unsound
  `Error::downcast_mut`). Both are now absent from `cargo audit`.
- **Dependency gate in CI**: add a RustSec advisory sweep (`cargo audit`) and a
  `cargo deny` license/bans/sources check (`deny.toml`) to the verification
  pipeline, and to `make verify`.

### Changed

- **Close the test-coverage gap in CI**: `tests/integration.rs`,
  `tests/ui_density_metrics.rs`, and `tests/ui_operator_walkthrough.rs` now run
  in the CI `Test` step and in `make test`/`make verify`, instead of being left
  to smoke only.

### Fixed

- **UI density walkthrough**: the comfortable-mode assertion now checks the
  node's genuine independent axes (`Process`, `Chain health`, `Height`) rather
  than the retired fused "Status Check" column, matching `tests/web.rs`.
- **4th parity gate**: every HTTP surface declared in `docs/AGENT_API.md`
  (as `### METHOD /path`) must be served by the router, with path parameters
  normalised so `{id}` and `{node_id}` match. The three earlier gates pin each
  surface the code names to its pair; this one anchors the automation protocol
  reference to the implementation, so a declared route the router does not
  serve fails the build. Verified by injecting a bogus endpoint.

### Docs

- Mark the root `NODE_MANAGER_*`, `PHASE*`, and `BENCHMARKS_STATUS.md`
  files as historical snapshots superseded by the custody/observation
  refactor, pointing to `claudedocs/NEONEXUS_GAP_REGISTER.md` as the live
  TODO/gap register.

### Refactor

- Split `src/web/node_form.rs` (760 lines) into `src/web/node_form.rs`
  (the `NodeDraft` model and its constructors/queries) plus
  `src/web/node_form/validate.rs` (the port/name/version rules that turn a
  parsed draft into a `NewNode`). The rules are a self-contained, purely
  internal concern and now read beside nothing else; the two modules keep
  their behaviour under the existing 26 node-form unit tests.
- Split `src/web/pages/nodes/iac_spec.rs` (703 lines) into
  `src/web/pages/nodes/iac_spec.rs` (the `IacFormat` dispatch and the in-page
  card) plus `src/web/pages/nodes/iac_spec/formats.rs` (the per-target IaC
  string builders). The format renderers are pure functions that produce
  Docker, K8s, CloudFormation, Terraform, CLI and JSON text; they now live
  apart from the table that dispatches to them. Behaviour is unchanged under
  the two existing iac_spec unit tests.

## [v4.3.1] - 2026-09-10

### Fixed

- **Node lifecycle event audit**: manual node launch failures in web/CLI paths are now recorded as `RuntimeLaunchedFailed` events, ensuring the Event Journal captures when an operator or automation initiates a Start that cannot complete due to workspace locks, binary mismatches, or port conflicts.

## [4.3.0] - 2026-09-10

This release expands the web workbench to reach actions that were previously
CLI/API-only, adds API token authentication for CI/automation, and resolves
findings from an Ultra CodeReview pass.

### Added

- **API token authentication for CI/automation** (P1-5): SHA-256-hashed bearer
  tokens with a `TokenPermission` enum, a two-layer `require_session` +
  `require_permission` middleware, and an `AuthIdentity` that distinguishes
  `Session` from `Token` callers, so automation can authenticate without a
  browser session while inheriting least-privilege permission scoping.
- **Prometheus metrics independent authentication route** (P0-1):
  `/api/metrics-prometheus` is now reachable with the `ReadFleet` permission
  rather than only a browser session, so an external scraper no longer has to
  authenticate as a browser does.
- **Web wallet profile import** (P0-2): `POST /wallets/import` with SHA-256
  validation and path-traversal protection.
- **Web wallet profile delete** (P2-3): `POST /wallets/{id}/delete`, recording a
  `NeoWalletProfileDeleted` event.
- **Web snapshot lifecycle events** (P1-1): save/verify/download/cache handlers
  now record `SnapshotSaved`, `SnapshotVerified`, `SnapshotDownloaded`, and
  `SnapshotCached` events.
- **Web plugin package installation** (P1-2): multipart upload processed on a
  background job with a 2GiB cap and SHA-256 verification, journaling a
  `PluginInstalled` event.
- **Web runtime smoke test trigger** (P1-3): `POST /nodes/{id}/smoke-test`
  records a `RuntimeSmokeTested` event.
- **Web log clear functionality** (P1-4): operators can clear a node's logs from
  the browser.
- **Runtime upgrade policy edit form** (P1-7): `POST /settings/runtime-upgrade`
  edits the policy from Settings, recording a `RuntimeUpgradePolicyUpdated`
  event.
- **`workspace_root` environment variable configuration** (P1-6): the workspace
  directory can now be set from the environment.
- **`ConfigApplied` and `NeoWalletProfileUsed` event producers** (P2-7).
- **Scheduled runtime upgrade execution** in the supervision engine: `LoopState::probe_runtime_upgrade()` periodically evaluates fleet-wide upgrade opportunities via `RuntimePackageManager::plan_catalog_fleet_upgrades()`, applies updates within configurable intervals and maintenance windows, automatically stopping running nodes before installation and restarting them post-upgrade. Policy configuration (`enabled`, `interval_minutes`, `max_nodes_per_run`, `maintenance_window`) takes effect without restarts; events are journaled to the Event Journal.

- **Web UI for Snapshot Apply**: POST handler `/snapshots/{snapshot_id}/apply/{node_id}` that verifies snapshot readiness, validates compatibility (network type, node type), calls `FastSyncSnapshotManager::apply_to_node()` to apply snapshots to compatible nodes, and returns flash-message feedback. Snapshots page now shows Apply buttons for verified, cached snapshots alongside compatible nodes.

- **Web UI for Backup Export**: POST handler `/backup/export` that triggers workspace backup exports using `WorkspaceBackupExporter`, displays a new `/backup` page with export status indicators and stat tiles (nodes, signer profiles, snapshots, events count), and records `BackupExported` audit events with artifact summary messages.

- **Private network magic override security fix**: Cross-replay prevention mechanism combining node identity binding, one-time token consumption, and monotonic generation counters with TTL enforcement. Magic override tokens are rejected when attempted on wrong nodes (`WrongNode`), expired (`Expired`), or replayed (`AlreadyConsumed`). Integrated into config export path to validate tokens before applying magic number changes.

- **60 unit tests for private_network module** covering scripts, verifier, exporter, validation support (wallets, sidecars), committee parsing, reports rendering, and signers endpoint handling — replacing placeholder assertions with behavioral coverage.

- **4 unit tests for supervision runtime upgrade scheduling** (`test_probe_runtime_upgrade_disabled_by_default`, `test_probe_runtime_upgrade_respects_interval`, `test_probe_runtime_upgrade_respects_maintenance_window`, `test_probe_runtime_upgrade_returns_early_without_catalog_config`) verifying policy gating logic without network dependencies.

### Changed

- **Runtime upgrade settings surface** changed from read-only facts to an
  editable form, paired with the `POST /settings/runtime-upgrade` handler.

### Fixed

- **Ultra CodeReview**: `apply_snapshot` now derives the workspace path from
  `data_dir` instead of the hardcoded `"workspaces"` directory.
- **Ultra CodeReview**: the plugin upload body limit uses `DefaultBodyLimit::max`
  as a 2GiB DoS safety net.
- **Ultra CodeReview**: failed runtime smoke-test events are raised to
  `Critical` severity for operator visibility.

## [4.2.0] — 2026-09-08

**Note:** This release was initially planned and documented; implementation completed in v4.1.x development cycle.

### Added

- Compact single-line inventory and fleet `node_row` anatomy (40pt slots)
  after geometry proof: status dot, name, type/net badges, RPC port, status pill.
- Headless operator walkthrough suite (`tests/ui_operator_walkthrough.rs`):
  Comfortable↔Compact chrome invariance, primary surface paint, density reload.
- Compact inventory page-fit unit gate and denser `list_row_frame` vertical
  margins for ≤40pt slots.
- Runtime Install `loading_callout` when package install/download work is in
  progress; disable Install/Download while busy.

### Changed

- `DensityMetrics::COMPACT` list heights: inventory/fleet **40 / 40** (was 44/56);
  journal empty slots remain **52**; chrome remains density-invariant **60 / 28 / 212**.
- Residual view spacing tokenised to `theme::XS` / `theme::SM` on several surfaces.

## [4.1.0] — 2026-09-06

### Added

- A named signer registry that can load `local-wallet`, loopback-HTTPS
  `local-signer`, and HTTPS `neo-os-service` profiles concurrently. Console,
  public-relay, and default internal-signing routes are selected independently;
  key identity is backend-qualified, selection never relies on URL guessing,
  and runtime failure never falls back across profiles.
- A process-local encrypted NEP-6 wallet signer for Neo N3/P-256. It pins and
  revalidates wallet identity, reads the passphrase only from a protected file,
  strictly parses the complete unsigned transaction, enables transaction
  signing by default, refuses consensus without durable anti-equivocation,
  keeps raw signing explicit and default-off, and is never exposed through the
  public relay or remote administration surfaces. The decrypted scalar is
  zeroizing but remains cached until the profile's last clone drops.
- The Rust signer-service integration split into configuration, secret-file,
  authentication, transport, admin/signing clients, and wire-model modules.
  It supports protected admin token files, distinct least-privilege internal
  signing identities, canonical Ed25519 workload assertions, mandatory
  authenticated TLS, and transparent relay of caller proofs and exact body
  bytes. Unix permissions and Windows owner/DACL validation fail closed.
- A dedicated Security / Signer workspace for key inventory, policy, caller and
  audit operations, with confirmation pages for destructive or credential-
  rotating actions. Key import and private-key/passphrase inputs remain outside
  the web boundary.
- A searchable Events surface and a redesigned operations-console information
  architecture: Overview, Fleet, Operations, Network, Assets, Security, and
  Workspace.
- Hash-pinned Content Security Policy and global no-store, anti-framing,
  anti-sniffing, referrer, permissions, COOP/CORP, and conditional HSTS headers.
- Main-branch build provenance through GitHub artifact attestations, alongside
  the existing explicitly pinned Ed25519 publisher-verification path.

### Changed

- Node Start, Stop, Restart, watchdog recovery, CLI control, and browser control
  now share a CAS-backed lifecycle protocol. Restart first proves and quiesces
  the recorded process; stop intent is durable before termination; PID identity,
  exit confirmation, and concurrent edits/deletes fail closed.
- Backup-restored runtime commands are quarantined until an operator explicitly
  rebinds a trusted local binary. Release ZIPs enforce entry, size, duplicate,
  path, and streaming digest limits; webhook redirects are disabled.
- Browser authentication accepts strong protected token files only in service
  mode, rate-limits login attempts, enforces exact-origin writes, scrubs control-
  plane secrets from child processes, and emits Secure cookies for HTTPS origins.
- The public federation API now exposes aggregate counts only; node identity,
  version, health, host, and process inventories require an operator session.
- Forms, tables, focus behavior, skip navigation, filter controls, labels, live
  regions, and mobile navigation were reworked for keyboard and screen-reader
  use. Background jobs no longer force a focus-destroying page refresh.

## [4.0.0] — 2026-08-28

The workbench is now a web service. One binary runs an HTTP server; operators
open the printed address in a browser. The desktop GUI is removed.

### Added

- **Web workbench** (`src/web/`): axum + tokio server, server-side rendered
  pages, and an embedded-assets policy (CSS/JS live in Rust string constants,
  so the source tree stays free of frontend files and the binary stays
  self-contained).
  - Pages: fifteen destinations in four sidebar groups — Fleet (Home, Nodes with
    per-node Start/Stop/Restart, Monitor, Logs), Operations (Readiness, Alerts,
    Federation, Private network roles), Assets (Runtimes, Plugins, Snapshots,
    Wallets, Config), and Insights (Metrics, Settings). Monitor, Logs, Plugins,
    Snapshots, Wallets, Federation, Roles, Config, and Settings had no browser
    surface before 4.0, and five of them had no CLI action at all, so removing
    the desktop shell had left them reachable only through the Rust API.
    Each reads through the `core::` facade — the same readiness, lifecycle,
    metrics, and catalogue calls the CLI makes.
  - Auth: single strong operator token from `--web-token-file` or
    `NEONEXUS_WEB_TOKEN_FILE` (interactive-only bootstrap on loopback; only its
    SHA-256 digest is retained), per-peer login throttling, exact-origin checks
    for protected writes, and an HttpOnly, SameSite=Strict session cookie with
    `Secure` on HTTPS deployments and 12-hour sliding expiry.
  - API: `/api/fleet`, `/api/readiness`, `/api/metrics-prometheus`,
    `/healthz`. Status badges poll every 5 s; all controls work without
    JavaScript (plain form posts + flash messages).
  - Lifecycle controls run the SAME core pipeline the CLI uses — readiness,
    managed config, supervised launch — so browser and script operators
    behave identically.
  - **Node manager**: register (`/nodes/new`), correct (`/nodes/{id}/edit`) and
    remove (`/nodes/{id}/delete`) nodes from the browser, which the removed
    desktop editor had been the only frontend for. A rejected save returns the
    operator's own text with the reason beside the field it belongs to.
    Validation borrows the domain's rules rather than restating them —
    `validate_node_ports`, `NodeType::supports_storage_engine`,
    `parse_argv_text` — and adds the two checks the workspace needs and the type
    system cannot see: a name already taken, and a port another node holds
    (including across the RPC/P2P pair, which would bind fine and then fail).
    Storage is only offered as a choice on clients that have one; the Neo X
    clients' embedded stores are stated, not presented as knobs. "Suggest free
    ports" asks the same planner the launch path uses, so it avoids both the
    fleet's ports and the host's. Deletion is a two-step flow naming what else
    goes with the node. Registration, updates and deletions are journaled as
    `node-created`, `node-updated` and `node-deleted` events.
- **Supervision engine** (`src/supervision.rs`): the background loop the
  workbench had been missing. The desktop shell's frame tick was what drained
  probe results, spawned interval probes, ran the watchdog and delivered alert
  webhooks; removing `src/app/` removed the heartbeat but not the settings that
  described it, so the workbench went on offering policies that nothing executed
  — the Alerts page showed a delivery history that could never grow, and a node
  that died stayed "Running" until someone looked. The engine now, on its own
  tick: reaps finished processes and journals the exit, restarts crashed ones
  within the watchdog policy, probes RPC health and federation peers on their
  configured intervals, routes qualifying journal entries to the webhook, and
  settles nodes that report Running without a process behind them. Policies are
  re-read each tick, so a change in Settings applies without a restart. It shares
  the server's one `ProcessSupervisor`, so a node the watchdog restarts is a node
  the browser can stop.
- Node launch and stop moved into that engine and the browser delegates to it,
  replacing two copies of the pipeline with one: a manual start and an automatic
  restart now take the same path against the same supervisor.
- `ProcessSupervisor` gained `disown_all`/`disown`, because its `Drop` terminates
  everything registered. A one-shot `--node-start` reported a node as launched and
  then killed it on the way out of `main`; it now hands the process over
  explicitly, and `--node-stop` reaches it by the pid the workspace recorded
  rather than only rewriting the row. Stopping no longer waits out a grace period
  on platforms where no graceful signal was actually sent.
- `src/health_events.rs`: the status-to-severity and status-to-wording helpers the
  engine needs, which had lived inside `src/app/` and were not about drawing a
  window.
- **Lifecycle and control audit trail.** Sweeping which event kinds still had a
  producer turned up 52 of 65 with none, and the ones that mattered were actions
  the workbench itself offers: Start, Stop and Restart wrote no journal entry, so
  "who stopped this node at 03:00" had no answer; neither did the plugin toggle,
  the workspace config export, or saving any of the four policies. All of them
  are journaled now, carrying the node they refer to so an entry is
  attributable rather than merely present. The remaining unproduced kinds belong
  to capabilities that still have no frontend and are listed under Known gaps
  rather than being quietly left out.
- **Startup reconciliation** in the engine: nodes left recorded as Running by a
  previous instance are checked against the host before anything is concluded.
  One whose process is still alive keeps its status — a workbench killed with
  SIGKILL leaves its nodes running, and clearing those rows would lose the only
  handle on them, so the next Start would launch a second node onto the same
  ports. One whose process is gone is settled and journaled as
  `RuntimeRecovered`. A pid answered by a *different* program is settled too but
  reported separately, because the number was recycled and the old process is
  not coming back. `Repository::clear_transient_runtime_state`, which had no
  production caller after the desktop shell went away, is deliberately not used
  here: it clears every Running row unconditionally, which is correct for an app
  that kills its own children on exit and wrong for a server.
- **Controlled runtime installation** (`/runtimes`): browse an enabled catalog
  profile, review a release, then install it. The review shows the catalogue and
  source it came from, the package platform beside this host's, the size limit,
  the expected digest, and whether a signer key is even configured — so an
  unsigned source cannot be presented as verified. Browsing reads the catalogue
  and writes nothing; the form carries only a profile id and a release id and the
  server re-resolves the URL, so the page cannot be pointed at an arbitrary host.
  A release built for another platform is refused before any bytes are
  transferred, an already-installed one is refused rather than silently
  replaced, and `RuntimePackageManager::install` checks digest, platform and
  signature before copying, so a verification failure leaves the host untouched.
- **Readable time** (`src/web/time.rs`): every table that showed a raw Unix
  second now shows `2026-08-29 12:47Z` with the elapsed reading beneath it
  ("3m ago"), and a machine-readable `<time datetime>`. Fixed-width so a column
  aligns, UTC because that is what the logs, events and probes are written in.
  The conversion is days-since-epoch arithmetic rather than another dependency,
  and is tested against independently known anchors and the leap-year rollovers
  where hand-rolled date code usually breaks.
- **Layout and interaction polish**: wide tables scroll instead of squeezing
  eight columns into slivers; a page with a job in flight refreshes itself when
  it finishes and stays put when idle; the flash and job panels announce through
  `aria-live`; the active nav item carries `aria-current`; headings keep their
  spacing whether or not they sit in a page header; federation probe history
  gained the breadcrumb the rest of the sub-pages use.
- **Background jobs** (`src/web/jobs.rs`): the install runs on its own thread
  behind a one-job-per-lane registry, so a multi-minute download cannot time out
  a browser, a reload still shows it running, and two concurrent installs cannot
  interleave writes into the same tree. The page reports state, result and
  failure reason.
- `--web` / `--bind` / `--port` / `--web-token-file` /
  `--web-public-origin` launch options. No options starts the web workbench (the
  default interactive experience).
- End-to-end web suite (`tests/web.rs`): real server on an ephemeral port,
  auth boundary, JSON API, node creation through the repository, and the
  stop-path persistence, all over plain HTTP.
- `make web-smoke` and a cross-platform web smoke step in CI.
- `docs/web.md` for the server, auth model, and cloud deployment.

### Changed

- `--source-quality` no longer enforces a 200-line budget on Rust source
  files; the maintenance-file budget (1000 lines) is unchanged.
- `README.md` rewritten for the web-first posture.

### Removed

- The native desktop application: `src/app/` (the egui/eframe shell, views,
  widgets, and theme layer), the `eframe`/`egui`/`egui-phosphor` and `image`
  dependencies, the ten `tests/ui_*` contract suites, and the
  `--native-ui-audit` gate with its CLI actions and CI steps.
- `--gui` (removed in favour of the default web experience; the flag now
  explains where the workbench went).

### Known gaps

The workbench reaches every surface the desktop shell offered, but not every
action on them:

- **Inventory pages are read-only where the action writes to the host or leaves
  the machine.** Snapshot import and apply, wallet profile import, delivering a
  real alert (the page previews routing only), and private-network
  materialisation remain CLI/API operations.
- **Several desktop capabilities still have no frontend at all**, discoverable
  as event kinds with no producer: role application (`RoleApplied`), plugin
  package install (`PluginInstalled`), the snapshot lifecycle (`SnapshotSaved`,
  `SnapshotVerified`, `SnapshotDownloaded`, `SnapshotCached`, `SnapshotApplied`),
  wallet profile import and deletion, private-network materialisation with its
  signer sidecars, backup export/import/validation, log clearing, and runtime
  smoke tests. Each is implemented in the core layer; none is reachable.
- **Scheduled runtime upgrades are not enforced.** The policy is stored,
  validated and displayed, and `RuntimePackageManager::plan_node_upgrade`,
  `plan_catalog_upgrade` and `plan_catalog_fleet_upgrades` all exist — but
  nothing calls them, so no node is ever upgraded on an interval. The Settings
  page now says so rather than implying the policy is live. The scheduler in
  `src/app/runtime_upgrade_policy/execution.rs` went away with the desktop
  shell and has not been replaced.
- `/api/metrics-prometheus` sits behind the session cookie, so an external
  Prometheus scraper must authenticate as a browser does or scrape through an
  internal route.
- TLS is not terminated in the binary; a reverse proxy is expected in front of
  the bound address.

## [3.1.0] — 2026-07-15

### Added

- Full v3.1 UI visual system (PR-01–15): theme density metrics scaffold,
  frozen kit (`list_row_frame`, `confirm_bar`, `page_chrome`, `busy_inline`),
  shell chrome tokens, nodes tab + density persistence (`appearance.ui_density`),
  Settings Storage density control with immediate Compact control metrics.
- `page_chrome` on all primaries and nested hubs; readiness/journal list matrix.
- Density geometry contracts (`tests/ui_density_geometry.rs`).

### Changed

- Home fleet always uses `node_row` matrix (no alternate grid selection geometry).
- Nodes Studio tools migrated to `ToolbarAction` toolbar.

## [3.0.0] — prior

- Six-primary information architecture, partial widget kit, god-state split,
  headless UI contract tests.

[4.3.0]: https://github.com/r3e-network/neo-nexus/compare/v4.2.0...v4.3.0
[4.2.0]: https://github.com/r3e-network/neo-nexus/compare/v4.1.0...v4.2.0
[4.0.0]: https://github.com/r3e-network/neo-nexus/compare/v3.1.0...v4.0.0
[3.1.0]: https://github.com/r3e-network/neo-nexus/compare/v3.0.0...v3.1.0
