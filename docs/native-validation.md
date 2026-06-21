# Native Rust App Validation

Validation date: 2026-06-21

This document describes the verification evidence expected before NeoNexus is
released or pushed as a pure Rust native application.

NeoNexus is validated as a desktop operations application, not as a frontend,
WebView, Tauri app, browser shell, or server wrapper. The gates cover Rust
correctness, cross-platform native CI, source purity, source quality, native UI
structure, node-management behavior, and release handoff evidence.

## Core Gates

```bash
cargo fmt --all -- --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test ci_policy
cargo test --test domain
cargo test --test repository
cargo run -- --self-check
cargo run -- --source-purity .
cargo run -- --source-quality .
cargo run -- --native-ui-audit .
cargo run -- --ci-policy .github/workflows/ci.yml
```

Expected result:

- Rust formatting is clean.
- The workspace compiles.
- Clippy emits no warnings with warnings denied.
- Library and named integration targets pass.
- The binary self-check exits successfully without opening the GUI.
- Source purity, source quality, native UI, and CI policy audits report zero
  findings.

If another Cargo job owns the default `target/` lock, run the same commands
with an isolated target directory:

```bash
env CARGO_TARGET_DIR=/tmp/neo-nexus-target CARGO_INCREMENTAL=0 cargo test --lib
```

## Native Application Boundary

The following commands make the native Rust boundary executable:

```bash
cargo run -- --source-purity .
cargo run -- --source-purity-json .
cargo run -- --source-quality .
cargo run -- --source-quality-json .
cargo run -- --native-ui-audit .
cargo run -- --native-ui-audit-json .
```

Expected result:

- No Node/Web manifests, frontend source files, `node_modules`, web/frontend
  directories, Docker/compose deployment files, nginx deployment files,
  WebView/Tauri project files, or WebView/Tauri dependencies are present.
- Production Rust has no panic-oriented development markers, document-style
  native layout markers, hardcoded platform shortcut labels, or oversized
  modules.
- Maintenance files, including Markdown and JSON docs, stay under the review
  budget enforced by the source quality scanner.
- The GUI starts through eframe/egui, defines fixed top/bottom/left/right and
  central panels, exposes explicit workspaces, sets minimum window sizing, and
  contains no WebView/Tauri/Wry or scroll-first UI markers.

## Manager And CLI Boundary

```bash
cargo run
cargo run -- --gui
cargo run -- --help
cargo run -- --self-check
```

Expected result:

- `cargo run` and `cargo run -- --gui` are application-mode entries.
- Headless options are dispatched only when an explicit CLI option is present.
- CLI help documents application mode and headless manager commands.
- `--gui` with extra positional arguments is rejected.
- Tests keep GUI mode out of the CLI action enum and verify the manager owns
  the startup split.

## neo-rs Gates

neo-rs is validated as a first-class runtime:

```bash
cargo run -- --runtime-smoke neo-rs /path/to/neo-node
cargo run -- --runtime-smoke-json neo-rs /path/to/neo-node
cargo run -- --generate-node-config neo-rs testnet rocksdb 10332 10333 /tmp/neo-rs.toml
cargo run -- --generate-node-config-json neo-rs testnet rocksdb 10342 10343 /tmp/neo-rs-json.toml
cargo run -- --validate-node-config neo-rs testnet rocksdb 10332 10333 /tmp/neo-rs.toml
cargo run -- --validate-node-config-json neo-rs testnet rocksdb 10332 10333 /tmp/neo-rs.toml
```

Expected result:

- Runtime smoke output records status, runtime kind, path, byte count, and
  SHA-256 evidence when the binary exists.
- Missing or blocked binaries return structured non-zero evidence.
- Generated neo-rs config is TOML.
- neo-rs storage posture is RocksDB-oriented.
- Managed config launch planning injects `--config <node.toml>` unless an
  operator-provided config flag is already present, in which case readiness
  reports a review finding instead of overwriting intent.

## Workspace And Operations Gates

```bash
cargo run -- --workspace-readiness /path/to/neonexus.db
cargo run -- --workspace-readiness-json /path/to/neonexus.db
cargo run -- --workspace-metrics /path/to/neonexus.db
cargo run -- --workspace-metrics-json /path/to/neonexus.db
cargo run -- --workspace-metrics-prometheus /path/to/neonexus.db
cargo run -- --workspace-integrity /path/to/neonexus.db
cargo run -- --workspace-integrity-json /path/to/neonexus.db
cargo run -- --export-readiness-report /path/to/neonexus.db /path/to/reports
cargo run -- --export-event-journal /path/to/neonexus.db /path/to/events
cargo run -- --export-support-bundle /path/to/neonexus.db /path/to/support
cargo run -- --export-support-bundle-json /path/to/neonexus.db /path/to/support-json
```

Expected result:

- Readiness findings include severity, stable key, resolution workspace,
  action label, and operator hint.
- Workspace integrity performs read-only SQLite integrity, foreign-key, schema,
  index, row-count, and page metadata checks.
- Metrics include system pressure, managed-node process data, uptime, and
  missing running-node PID evidence.
- Support bundles contain redacted diagnostics, metrics text/JSON/Prometheus
  snapshots, readiness, bounded event history, node inventory, privacy note,
  manifest, and ZIP archive.
- Support bundles do not include raw databases, raw runtime logs, private keys,
  wallet passwords, passphrases, mnemonics, seeds, bearer values, API keys,
  tokens, webhook secrets, cached runtime packages, or snapshots.

## Config, Backup, Wallet, And Launch Pack Gates

```bash
cargo run -- --export-node-configs /path/to/neonexus.db /path/to/configs
cargo run -- --export-node-configs-json /path/to/neonexus.db /path/to/configs-json
cargo run -- --export-backup /path/to/neonexus.db /path/to/backups
cargo run -- --export-backup-json /path/to/neonexus.db /path/to/backups-json
cargo run -- --validate-backup /path/to/neonexus-backup.json
cargo run -- --validate-backup-json /path/to/neonexus-backup.json
cargo run -- --import-backup /path/to/neonexus.db /path/to/neonexus-backup.json
cargo run -- --import-backup-json /path/to/neonexus.db /path/to/neonexus-backup.json
cargo run -- --validate-wallet /path/to/validator.wallet.json
cargo run -- --validate-wallet-json /path/to/validator.wallet.json
cargo run -- --import-wallet-profile /path/to/neonexus.db /path/to/validator.wallet.json validator-wallet "Validator wallet"
cargo run -- --validate-launch-pack /path/to/private-network/manifest.json
cargo run -- --launch-pack-sidecars /path/to/private-network/manifest.json
cargo run -- --launch-pack-sidecars-json /path/to/private-network/manifest.json
```

Expected result:

- Config export writes runtime-specific files under collision-safe directories.
- Backup import validates before restore, refuses active-node target
  workspaces, and restores nodes as stopped.
- Wallet validation checks encrypted NEP-6 structure, Base58Check addresses,
  NEP-2 key shape, scrypt parameters, contract public keys, and
  address/script-hash consistency.
- Wallet profile import persists metadata only.
- Launch pack validation checks manifest invariants, managed configs, scripts,
  sidecar command plans, signer wallet references, signer endpoints, encrypted
  wallet structure, and secret-boundary rules.

## Runtime Catalog And Snapshot Gates

Runtime catalog examples:

```bash
cargo run -- --source-quality docs
```

Runtime Manager validation should also be exercised through domain tests and
native workflow tests:

- local catalog import,
- signed HTTPS catalog import,
- HTTPS-only redirect handling,
- max-byte enforcement,
- SHA-256 package verification,
- optional detached Ed25519 signature enforcement,
- installed runtime registry filtering,
- selected-node and fleet catalog upgrade planning,
- running-node upgrade restart readiness.

Fast Sync validation should cover:

- local and signed HTTPS snapshot catalog import,
- SHA-256 cache verification,
- archive staging before publication,
- unsafe path and symlink rejection,
- stopped-node data directory import,
- neo-rs RocksDB snapshot posture.

## Release Gates

```bash
cargo build --release
target/release/neo-nexus --package-release dist
target/release/neo-nexus --verify-release-package dist
target/release/neo-nexus --verify-release-package-json dist
```

Expected result:

- Release build succeeds.
- Packaging creates a platform ZIP, sidecar manifest, and `.sha256` checksum.
- Verification checks sidecar manifest, checksum file, archive hash, ZIP
  manifest, ZIP contents, and packaged binary hash.
- Verification failure exits non-zero and emits text/JSON evidence.

## CI Policy

```bash
cargo run -- --ci-policy .github/workflows/ci.yml
cargo run -- --ci-policy-json .github/workflows/ci.yml
```

Expected result:

- Ubuntu, macOS, and Windows runners are present.
- CI runs format, check, clippy, library tests, named integration tests,
  self-check, source purity, source quality, native UI audit, CI policy audit,
  runtime smoke, RPC health, workspace readiness, metrics, integrity, reports,
  support bundles, event journal, config, backup, wallet, launch-pack, and
  release package checks.
- CI does not reintroduce frontend, Node, WebView, or Tauri tooling.

## Current Validation Evidence

On 2026-06-21, the following local gates were run before the current
documentation refresh:

- `cargo fmt --all -- --check`
- `git diff --check`
- `CARGO_INCREMENTAL=0 cargo check --quiet`
- `CARGO_INCREMENTAL=0 cargo clippy --quiet --all-targets -- -D warnings`
- `CARGO_INCREMENTAL=0 cargo test --quiet`
- `cargo run --quiet -- --source-quality /Users/jinghuiliao/git/r3e/neo-nexus`
- `cargo run --quiet -- --source-purity /Users/jinghuiliao/git/r3e/neo-nexus`
- `cargo run --quiet -- --native-ui-audit /Users/jinghuiliao/git/r3e/neo-nexus`
- `cargo run --quiet -- --ci-policy /Users/jinghuiliao/git/r3e/neo-nexus/.github/workflows/ci.yml`

The final push gate should repeat the core gates after documentation changes.
