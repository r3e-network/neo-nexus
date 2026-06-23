# NeoNexus

NeoNexus is a pure Rust native application for Neo N3 node operations. It is
built with Rust, `eframe`/`egui`, SQLite, and reusable Rust domain services.

This repository is not a frontend project, browser shell, WebView, Tauri app,
or server-container wrapper. The default experience is a desktop operations
workbench with fixed inventory, workspace, inspector, toolbar, menu, and status
regions. Long operational data is handled with filters, paging, and focused
detail panels instead of a scroll-first web page layout.

## What Operators Can Do

- Manage neo-cli, neo-go, and neo-rs node definitions from one native app.
- Launch, stop, restart, reconcile, and inspect supervised node processes.
- Generate, validate, and export neo-cli JSON, neo-go YAML, and neo-rs TOML
  configuration.
- Run runtime smoke checks, RPC health checks, readiness checks, workspace
  integrity checks, metrics exports, backup validation, wallet validation, and
  release package verification without opening the GUI.
- Install verified runtime packages, import runtime catalogs, plan catalog
  upgrades, and roll selected or fleet upgrades through native controls.
- Export support bundles, readiness reports, event journals, backups, node
  configs, private-network launch packs, and release artifacts with redacted
  evidence suitable for handoff.

neo-rs is a first-class runtime target. NeoNexus recognizes the `neo-node`
binary, validates RocksDB-oriented TOML configs, supports Fast Sync snapshot
catalog entries, uses the same runtime catalog machinery as the other Neo
runtimes, and routes neo-rs readiness findings into the same Operations
workflow used for neo-cli and neo-go.

## Requirements

- Rust 1.91 or newer.
- Linux, macOS, or Windows with the platform GUI dependencies required by
  `eframe`.
- Optional node binaries if you want to start real processes:
  `neo-cli`, `neo-go`, or neo-rs `neo-node`.

Linux development packages used by CI include ALSA, Fontconfig, X11, cursor,
keyboard, RandR, and OpenGL development headers.

## Run The Native Application

```bash
cargo run
cargo run -- --gui
```

No option and `--gui` both enter the native desktop workbench through the
manager layer. Any other option is an explicit headless manager command and
must not accidentally launch the GUI.

Useful headless examples:

```bash
cargo run -- --self-check
cargo run -- --runtime-smoke neo-rs /path/to/neo-node
cargo run -- --runtime-smoke-json neo-rs /path/to/neo-node
cargo run -- --rpc-health 127.0.0.1:10332
cargo run -- --workspace-readiness /path/to/neonexus.db
cargo run -- --workspace-metrics-json /path/to/neonexus.db
cargo run -- --workspace-metrics-prometheus /path/to/neonexus.db
cargo run -- --workspace-integrity-json /path/to/neonexus.db
cargo run -- --generate-node-config neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --validate-node-config neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --export-support-bundle /path/to/neonexus.db /path/to/support
cargo run -- --validate-wallet /path/to/validator.wallet.json
cargo run -- --validate-launch-pack /path/to/private-network/manifest.json
```

After a release build:

```bash
cargo build --release
target/release/neo-nexus --package-release dist
target/release/neo-nexus --verify-release-package dist
target/release/neo-nexus --verify-release-package-json dist
```

## Verify

```bash
cargo fmt --all -- --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test ci_policy
cargo test --test domain
cargo test --test repository
cargo run -- --source-purity .
cargo run -- --source-quality .
cargo run -- --native-ui-audit .
cargo run -- --ci-policy .github/workflows/ci.yml
```

`make verify` runs the broader local gate set, including smoke checks for
runtime probes, alerts, readiness, metrics, integrity, support bundles, event
journals, node config export/generation, backups, wallets, launch packs, and
release-adjacent flows.

The desktop binary is excluded from Cargo's default test harness. Tests run
through the library and named integration targets so filtered test commands
and target listing do not open the native application.

## Native Workbench

The application is organized around repeatable operations rather than pages:

- Overview and Inventory keep filtered node lists paged and selection-aware.
- Node Studio owns node editing, launch previews, runtime args, and port
  planning.
- Monitor shows RPC health, resource pressure, and managed-process state.
- Logs gives fixed-panel runtime log search, tail-follow, and diagnosis.
- Operations owns readiness triage, action queues, port conflicts, workspace
  integrity, support bundles, event history, backups, and release evidence.
- Runtime Manager handles verified runtime packages, catalogs, installed
  runtime inventory, signed catalog trust, Fast Sync snapshots, and upgrades.
- Wallet Profiles stores encrypted Neo wallet metadata only, never private
  keys, wallet bytes, passwords, passphrases, mnemonics, or seeds.
- Roles and private-network tools plan validator topologies and export launch
  packs with explicit signer references and no-shell sidecar command plans.
- Settings owns upgrade policy, release packaging, audit controls, and native
  application preferences.

All of these surfaces use the same command model for menus, toolbar controls,
and keyboard accelerators. Platform shortcut labels are generated by the
native shortcut layer so macOS, Linux, and Windows users see familiar command
text.

The shell presents a grouped navigation sidebar (Workspace, Nodes, Network,
System) alongside a fixed top bar, status bar, node inventory, and an optional
inspector. Dense surfaces use an in-page segmented control to focus on one
region at a time instead of crowding several panels onto one screen. A shared
design-token layer (type scale, spacing, and semantic status colours) gives
every view a consistent light or dark macOS-style appearance, and the active
view, inspector visibility, and theme are remembered across restarts.

## Architecture

The source tree is intentionally Rust-only:

- `src/main.rs` is a thin binary entrypoint.
- `src/manager/` classifies startup arguments into native GUI mode or
  explicit headless manager commands.
- `src/app/` contains the native `egui` application shell, the design-token
  theme layer, shared widgets, view modules, and workflow bindings.
- `src/core/` is the UI-free facade shared by GUI and CLI surfaces.
- `src/cli/` parses headless commands and renders text/JSON output.
- Domain modules such as `runtime`, `snapshots`, `config`, `launch`,
  `repository`, `backup`, `wallet`, `private_network`, `supervisor`,
  `source_purity`, `source_quality`, `native_ui`, and `ci_policy` hold
  reusable behavior outside the application shell.

Tests are kept out of `src/` so the source tree reads as production only:

- `tests/unit/` mirrors the `src/` module layout and holds the in-crate unit
  tests. Each production module keeps a one-line `#[cfg(test)] #[path = ...]
  mod tests;` stub that points at its `tests/unit/` file, so the tests retain
  private access while their code lives outside `src/`.
- `tests/domain`, `tests/ci_policy`, and `tests/repository` hold the public-API
  integration tests compiled as separate test crates.

- `--source-purity` rejects Node/Web manifests, frontend source files,
  `node_modules`, web/frontend directories, Docker/compose and nginx deployment
  artifacts, WebView/Tauri project files, and WebView/Tauri dependencies.
- `--source-quality` rejects oversized Rust modules, oversized maintenance
  files, panic-oriented production markers, hardcoded platform shortcut labels,
  and document-style native layout markers.
- `--native-ui-audit` requires eframe/egui startup, minimum window sizing,
  fixed top/bottom/left/right/central panels, explicit workspaces, and no
  WebView/Tauri/Wry or scroll-first UI markers.
- `--ci-policy` verifies cross-platform native CI coverage on Ubuntu, macOS,
  and Windows with native gates and no frontend toolchain.

## Documentation

- [Native Rust App](docs/native-rust.md) explains architecture, application
  mode, runtime support, release packaging, catalogs, snapshots, and private
  network behavior.
- [Native Rust App Validation](docs/native-validation.md) records the gates and
  release evidence expected before handoff.
- [Operator Benchmarks](docs/operator-benchmarks.md) summarizes the node
  manager product patterns used to shape the workbench.
- [Runtime catalog example](docs/runtime-catalog.example.json) and
  [snapshot catalog example](docs/snapshot-catalog.example.json) are importable
  schema samples for Runtime Manager and Fast Sync workflows.

## Current Gaps

The native Rust conversion is broad, but further production hardening should
continue in these areas:

- More Linux and Windows smoke runs against real neo-cli, neo-go, and neo-rs
  binaries.
- More long-running process-supervision tests with real node data directories.
- Signed catalog and release-distribution exercises with real operator keys.
- Additional accessibility and keyboard-only workflow review on each desktop
  platform.
