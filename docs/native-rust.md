# Native Rust App

NeoNexus is a pure Rust desktop application for Neo N3 node operations. It is
implemented with Rust, `eframe`/`egui`, SQLite, and reusable domain services.

The project deliberately avoids WebView, Tauri, browser-shell, frontend, and
server-container application models. The operator experience is a fixed-panel
desktop workbench: inventory on the left, task workspace in the center,
inspector and operational detail on the side, persistent toolbar/menu commands,
and status feedback at the bottom. Long lists are handled with bounded paging,
filters, and focused detail panes.

## Application Mode

```bash
cargo run
cargo run -- --gui
```

Both commands enter the native GUI through `src/manager/`. Every other option
is treated as an explicit headless command, which keeps CI, automation, and
release checks deterministic.

The split is enforced in code:

- `src/main.rs` delegates to the manager.
- `src/manager/planner.rs` classifies default GUI mode, explicit `--gui`, and
  CLI actions.
- `src/cli/` no longer owns a fake GUI action; it owns headless parsing and
  output only.
- Tests cover default GUI startup, explicit GUI startup, invalid `--gui`
  argument combinations, CLI help, and source organization.

## Operator Flow

1. Create or import node definitions for neo-cli, neo-go, or neo-rs.
2. Validate binaries, runtime versions, generated configs, ports, storage
   posture, plugins, wallets, and launch readiness.
3. Start, stop, restart, reconcile, or upgrade supervised native processes.
4. Triage readiness, RPC health, logs, metrics, port conflicts, backup safety,
   workspace integrity, and event history in Operations.
5. Export support bundles, reports, backups, configs, launch packs, runtime
   evidence, and release packages for handoff.

## Native Workspaces

- Overview: paged fleet view, selected-node summary, and quick lifecycle state.
- Inventory: searchable, status-filtered node registry.
- Node Studio: node model editing, port planning, launch previews, managed
  config posture, and runtime args.
- Monitor: non-blocking RPC health probes, system pressure, managed-process
  metrics, missing PID focus, and reconciliation cues.
- Logs: fixed-panel log tail, search, follow, and diagnosis.
- Operations: readiness action queue, selected-node checks, port matrix,
  workspace safety, integrity checks, event journal, backups, reports, support
  bundles, and release evidence.
- Runtime Manager: runtime catalog profiles, trusted signers, verified runtime
  package install/download, installed runtime inventory, Fast Sync snapshots,
  upgrade planning, and fleet rollout policy.
- Wallet Profiles: encrypted NEP-6 wallet metadata validation and signer
  reference handoff without storing secret material.
- Roles: node role postures, plugin separation, private-network topology
  planning, signer references, and launch pack export.
- Settings: runtime upgrade policy, release packaging, command preferences, and
  audit controls.

## Runtime Support

NeoNexus treats three Neo node runtimes as production paths:

- neo-cli: JSON config generation, plugin package provenance, direct binary or
  `dotnet Neo.CLI.dll` recognition, managed workdirs, and plugin inventory.
- neo-go: YAML config generation, LevelDB posture, explicit config flag review,
  runtime smoke probes, and managed launch config injection.
- neo-rs: `neo-node` recognition, TOML config generation/validation, RocksDB
  posture, managed `--config` injection, runtime smoke probes, catalog upgrade
  planning, Fast Sync entries, and private-network validator config posture.

Runtime smoke probes use bounded `--version` or `--help` style checks and
capture redacted stdout/stderr, binary path, byte count, and SHA-256 evidence.

## Headless Commands

Headless commands support CI and operator automation without opening the GUI:

```bash
target/release/neo-nexus --version
target/release/neo-nexus --self-check
target/release/neo-nexus --runtime-smoke-json neo-rs /path/to/neo-node
target/release/neo-nexus --rpc-health-json 127.0.0.1:10332
target/release/neo-nexus --workspace-readiness-json /path/to/neonexus.db
target/release/neo-nexus --workspace-metrics-prometheus /path/to/neonexus.db
target/release/neo-nexus --workspace-integrity-json /path/to/neonexus.db
target/release/neo-nexus --export-support-bundle-json /path/to/neonexus.db /path/to/support
target/release/neo-nexus --export-event-journal /path/to/neonexus.db /path/to/events
target/release/neo-nexus --generate-node-config-json neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
target/release/neo-nexus --validate-node-config-json neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
target/release/neo-nexus --validate-wallet-json /path/to/validator.wallet.json
target/release/neo-nexus --validate-launch-pack /path/to/private-network/manifest.json
target/release/neo-nexus --launch-pack-sidecars-json /path/to/private-network/manifest.json
target/release/neo-nexus --package-release dist
target/release/neo-nexus --verify-release-package-json dist
```

Text output is for operators; JSON output is for automation and release
collectors. Non-zero exit codes are used for blocked or failed validation
states where automation should stop.

## Release Packaging

Release packaging is available from Settings and from the CLI:

```bash
cargo build --release
target/release/neo-nexus --package-release dist
target/release/neo-nexus --verify-release-package dist
target/release/neo-nexus --verify-release-package-json dist
```

Packaging writes a platform ZIP, sidecar JSON manifest, and `.sha256`
checksum. Verification checks the sidecar manifest, checksum file, archive
hash, ZIP contents, packaged binary hash, and embedded release manifest.

## Source Layout

```text
src/
  main.rs                 thin native binary entrypoint
  manager/                GUI-vs-headless startup classification
  app/                    native eframe/egui shell, views, and workflows
  core/                   UI-free facade shared by GUI and CLI
  cli/                    headless parser and text/JSON renderers
  repository.rs           SQLite workspace persistence
  runtime/                runtime catalogs, packages, signatures, upgrades
  snapshots/              Fast Sync manifests, catalogs, cache, import
  config/                 neo-cli JSON, neo-go YAML, neo-rs TOML
  launch.rs               runtime-specific launch plans
  supervisor.rs           native managed-process lifecycle
  wallet/                 encrypted Neo wallet validation and metadata import
  private_network/        role materialization and launch pack export
  source_purity.rs        executable Rust-only repository boundary
  source_quality.rs       module and production-source quality gate
  native_ui.rs            executable native application UI audit
  ci_policy.rs            cross-platform native CI policy audit
```

The application shell consumes shared behavior through `src/core/` and
`src/app/domain.rs`. CLI actions and output renderers also consume the same
domain facade, which keeps native and headless behavior consistent.

## Data And Evidence

The local SQLite workspace stores node definitions, plugin inventory, runtime
events, remote federation profiles, runtime catalog profiles, trusted signer
profiles, Fast Sync metadata, probe history, alert history, and settings.

Evidence exports include:

- readiness text/JSON reports,
- event journal text/JSON reports,
- support bundles with redacted diagnostics and ZIP manifests,
- workspace backups with restore validation,
- node config export/generation/validation reports,
- wallet validation reports,
- launch pack validation reports,
- workspace integrity and metrics reports,
- release package verification reports.

Support bundles are diagnostics, not backups. They exclude raw databases, raw
runtime logs, private keys, wallet passwords, passphrases, mnemonics, seeds,
authorization values, API keys, tokens, webhook secrets, runtime packages, and
snapshot caches.

## Runtime Catalogs

Runtime Manager imports local JSON catalogs or signed HTTPS catalogs. The
schema stays intentionally small:

- `schema_version`: `1`.
- `generated_at_unix`: optional generation timestamp.
- `releases`: runtime package entries.
- `node_type`: `neo-cli`, `neo-go`, or `neo-rs`.
- `platform.os` and `platform.arch`: host compatibility filters.
- `url`, `file_name`, `executable_name`, `expected_sha256`, and `max_bytes`:
  download and verification metadata.

See `docs/runtime-catalog.example.json`.

Downloads require HTTPS, only follow HTTPS redirects, enforce size limits,
verify SHA-256, optionally enforce detached Ed25519 signatures, and publish
packages atomically into the managed runtime cache.

## Fast Sync Snapshots

Fast Sync catalogs use the same trust posture:

- local or signed HTTPS catalog source,
- schema version `1`,
- explicit network and runtime,
- HTTPS source URL,
- expected SHA-256,
- maximum byte budget,
- native cache verification before publication.

See `docs/snapshot-catalog.example.json`.

Snapshot archive import is handled by Rust code. It rejects unsafe paths and
symbolic links, uses staging before publication, and refuses to overwrite
existing chain data silently.

## Private Networks

The Roles workspace can materialize one-node, four-node, or seven-node private
network templates for neo-cli, neo-go, or neo-rs. Launch packs include managed
configs, deterministic seed lists, network magic, validator count, committee
public key references, wallet provisioning evidence, platform scripts, signer
sidecar command templates, no-shell argv execution plans, and a runbook.

NeoNexus never invents validator private keys. Launch packs use references to
operator-provided encrypted wallets and signer endpoints. Validation rejects
wallet provisioning files or command plans that contain inline password,
private-key, mnemonic, seed, token, or other secret markers.

## Executable Boundaries

The project includes local and CI gates that make the native Rust contract
observable:

```bash
cargo run -- --source-purity .
cargo run -- --source-quality .
cargo run -- --native-ui-audit .
cargo run -- --ci-policy .github/workflows/ci.yml
```

`source-purity` rejects Node/Web/frontend/WebView/Tauri/server-container
artifacts. `source-quality` rejects oversized modules and production markers
that do not belong in a professional native application. `native-ui-audit`
requires the fixed-panel eframe/egui shell. `ci-policy` requires Ubuntu,
macOS, and Windows verification without frontend tooling.
