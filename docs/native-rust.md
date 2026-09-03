# The Pure Rust Workbench

NeoNexus is a pure Rust application for Neo N3 node operations. It is
implemented with Rust, axum/tokio (the web workbench server), SQLite, and
reusable domain services.

The project deliberately avoids WebView, Tauri, Node toolchains, frontend
frameworks, and browser-shell application models. The browser UI is
server-rendered by the same binary that runs the domain services: fixed
navigation, a fleet overview, node studios, and evidence exports. Long lists
are handled with bounded paging, filters, and focused detail pages.

## Application Mode

```bash
cargo run                       # start the web workbench on 127.0.0.1:8080
cargo run -- --web --bind 0.0.0.0 --port 8080 --web-token "$TOKEN"
```

The default experience is the web workbench through `src/manager/`. Every
other option is treated as an explicit headless command, which keeps CI,
automation, and release checks deterministic.

The split is enforced in code:

- `src/main.rs` delegates to the manager.
- `src/manager/planner.rs` classifies default web mode, explicit `--web` with
  its launch options, and CLI actions.
- `src/cli/` owns headless parsing and output only.
- Tests cover default web startup, web launch-option parsing, the removed
  `--gui` error path, CLI help, and source organization.

See `docs/web.md` for the server, the auth model, and cloud deployment.

## Operator Flow

1. Create or import node definitions for neo-cli, neo-go, or neo-rs.
2. Validate binaries, runtime versions, generated configs, ports, storage
   posture, plugins, wallets, and launch readiness.
3. Start, stop, restart, or inspect supervised native processes — from the
   browser or the CLI; both drive the same core pipeline.
4. Triage readiness, RPC health, metrics, backup safety, workspace integrity,
   and event history in Operations.
5. Export support bundles, reports, backups, configs, launch packs, runtime
   evidence, and release packages for handoff.

## Workbench Surfaces

- Home: fleet counts, host CPU/memory pressure, and the fleet table with live
  status polling.
- Nodes: node list and the per-node studio — config facts, RPC health trend,
  and lifecycle controls.
- Operations: readiness evaluation, the runtime event journal, and evidence
  exports.
- Metrics: workspace metrics snapshot and the Prometheus exposition.

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

Headless commands support CI and operator automation without opening a
browser:

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
target/release/neo-nexus --node-start /path/to/neonexus.db "node name"
target/release/neo-nexus --package-release dist
target/release/neo-nexus --verify-release-package-json dist
```

Text output is for operators; JSON output is for automation and release
collectors. Non-zero exit codes are used for blocked or failed validation
states where automation should stop.

## Release Packaging

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
  main.rs                 thin binary entrypoint
  manager/                web-vs-headless startup classification
  web/                    axum workbench: router, auth, pages, API, assets
  core/                   UI-free facade shared by the web workbench and CLI
  cli/                    headless parser and text/JSON renderers
  repository.rs           SQLite workspace persistence
  runtime/                runtime catalogs, packages, signatures, upgrades
  snapshots/              Fast Sync manifests, catalogs, cache, import
  config/                 neo-cli JSON, neo-go YAML, neo-rs TOML
  launch.rs               runtime-specific launch plans
  supervisor.rs           native managed-process lifecycle
  wallet/                 encrypted Neo wallet validation and metadata import
  signer_client/          client-only NeoOS custody API and workload authentication
  private_network/        role materialization and launch pack export
  source_purity.rs        executable Rust-only repository boundary
  source_quality.rs       production-source quality gate
  ci_policy.rs            cross-platform CI policy audit
```

The web workbench consumes shared behavior through `src/core/`. CLI actions
and output renderers consume the same domain facade, which keeps browser and
headless behavior consistent.

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

Runtime catalogs are imported as local JSON files or signed HTTPS catalogs.
The schema stays intentionally small:

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

Launch packs can materialize one-node, four-node, or seven-node private
network templates for neo-cli, neo-go, or neo-rs. Launch packs include managed
configs, deterministic seed lists, network magic, validator count, committee
public key references, wallet provisioning evidence, platform scripts, signer
sidecar command templates, no-shell argv execution plans, and a runbook.

NeoNexus never invents validator private keys. Launch packs use references to
operator-provided encrypted wallets and signer endpoints. Validation rejects
wallet provisioning files or command plans that contain inline password,
private-key, mnemonic, seed, token, or other secret markers.

## Executable Boundaries

The project includes local and CI gates that make the pure Rust contract
observable:

```bash
cargo run -- --source-purity .
cargo run -- --source-quality .
cargo run -- --ci-policy .github/workflows/ci.yml
```

`source-purity` rejects Node/Web/frontend/WebView/Tauri artifacts — browser
assets live in `src/web/assets.rs` as Rust string constants for exactly this
reason. `source-quality` rejects production markers that do not belong in a
professional codebase, plus oversized repository maintenance files.
`ci-policy` requires Ubuntu, macOS, and Windows verification without frontend
tooling.
