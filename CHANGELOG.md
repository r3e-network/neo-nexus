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

The web workbench now covers every surface the desktop shell offered, but it is
not a full replacement for it:

- **Node registration has no frontend.** Create and delete lived in
  `src/app/node_lifecycle_flow/editor/`, so `Repository::create_node` still has
  no production caller and a fresh workspace cannot gain its first node except
  by restoring a backup (`--import-backup`) or through the Rust API. The Home
  empty state says so rather than pretending otherwise.
- **Inventory pages are read-only where the action writes to the host or leaves
  the machine.** Runtime download and install, snapshot import and apply, wallet
  profile import, delivering a real alert (the page previews routing only), and
  private-network materialisation remain CLI/API operations.
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
