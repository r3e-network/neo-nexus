# Changelog

All notable changes to NeoNexus are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
  - Auth: single operator token (`--web-token`, `NEONEXUS_WEB_TOKEN`, or
    generated and printed at startup; only its SHA-256 digest is kept in
    memory), HttpOnly session cookie with 12-hour sliding expiry, redirect to
    login for pages and 401 for the API.
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
- **Background jobs** (`src/web/jobs.rs`): the install runs on its own thread
  behind a one-job-per-lane registry, so a multi-minute download cannot time out
  a browser, a reload still shows it running, and two concurrent installs cannot
  interleave writes into the same tree. The page reports state, result and
  failure reason.
- `--web` / `--bind` / `--port` / `--web-token` launch options. No options
  starts the web workbench (the default experience).
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
- `/api/metrics-prometheus` sits behind the session cookie, so an external
  Prometheus scraper must authenticate as a browser does or scrape through an
  internal route.
- TLS is not terminated in the binary; a reverse proxy is expected in front of
  the bound address.

## [3.2.0] — 2026-07-15

### Added

- Compact **single-line** inventory and fleet `node_row` anatomy (40pt slots)
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

[4.0.0]: https://github.com/r3e-network/neo-nexus/compare/v3.2.0...v4.0.0
[3.2.0]: https://github.com/r3e-network/neo-nexus/compare/v3.1.0...v3.2.0
[3.1.0]: https://github.com/r3e-network/neo-nexus/compare/v3.0.0...v3.1.0
