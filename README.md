# NeoNexus

NeoNexus is a pure Rust operations workbench for Neo N3 node fleets. One
binary starts a **web workbench**: open the printed address in a browser and
operate your node fleet from anywhere — a laptop, a bastion host, or a cloud
server. The same binary also exposes the full headless CLI for scripts and CI.

There is no desktop application since 4.0.0. The workbench runs as an HTTP
service (axum + tokio), renders server-side, and ships its browser assets
inside the binary — no Node toolchain, no external services, one executable.

## What Operators Can Do

- Manage neo-cli, neo-go, neo-rs, and Neo X node definitions from the browser
  or the CLI.
- Launch, stop, restart, and inspect supervised node processes through the
  shared core pipeline (readiness → managed config → supervise → persist).
- Run runtime smoke checks, RPC health checks, readiness checks, workspace
  integrity checks, metrics exports, backup validation, wallet validation, and
  release package verification without opening a browser.
- Import runtime catalogs, validate NEP-6 wallet profiles (metadata only), and
  validate private-network launch packs.

neo-rs is a first-class runtime target. NeoNexus recognizes the `neo-node`
binary, validates RocksDB-oriented TOML configs, supports Fast Sync snapshot
catalog entries, and routes neo-rs readiness findings into the same Operations
workflow used for neo-cli and neo-go.

## Requirements

- Rust 1.91 or newer.
- Linux, macOS, or Windows.
- Optional node binaries if you want to start real processes:
  `neo-cli`, `neo-go`, or neo-rs `neo-node`.

Linux development packages used by CI include ALSA, Fontconfig, X11, cursor,
keyboard, RandR, and OpenGL development headers (the GUI toolkit is gone, but
transitive skia/geometry crates in the tree may still expect them until the
4.x dependency audit lands — CI installs them today).

## Run The Web Workbench

```bash
cargo run
```

No options starts the workbench server on `127.0.0.1:8080` and prints the
address plus a generated sign-in token. Open the address in a browser and sign
in with the token.

Cloud-shaped options:

```bash
cargo run -- --web --bind 0.0.0.0 --port 8080 --web-token "$NEONEXUS_WEB_TOKEN"
```

- `--bind` defaults to `127.0.0.1`; set `0.0.0.0` on a cloud host behind a
  TLS-terminating reverse proxy.
- `--web-token` sets the operator token explicitly; otherwise the
  `NEONEXUS_WEB_TOKEN` environment variable is used; otherwise a one-off token
  is generated and printed at startup. Only the SHA-256 digest is kept in
  memory.
- Sessions are HttpOnly cookies with a 12-hour sliding expiry.

The workspace database lives at `NEONEXUS_DATA_DIR/neonexus.db` (or the OS data
directory), the same file the CLI writes to — browser operators and scripted
operators see the same workspace.

## Workbench Surfaces

| Page | What operators do there |
|------|--------------------------|
| **Home** | Fleet counts, host CPU/memory pressure, fleet table with live status polling |
| **Nodes** | Node list, per-node config facts, RPC health trend, Start/Stop/Restart controls |
| **Operations** | Fleet readiness evaluation and the runtime event journal |
| **Metrics** | Workspace metrics snapshot and the Prometheus exposition |

Status badges poll `/api/fleet` every 5 seconds; every control also works
without JavaScript (plain form posts with flash messages). `/healthz` is a
public liveness endpoint for load balancers. `/api/metrics-prometheus` serves
the same Prometheus exposition the CLI exports.

## Headless CLI

All operational commands run without the workbench and share its core
pipeline:

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

Node control uses the same readiness + launch path the web workbench uses, so
a scripted node and a browser-operated node behave identically:

```bash
cargo run -- --node-list    /path/to/neonexus.db
cargo run -- --node-status  /path/to/neonexus.db "node name"
cargo run -- --node-start   /path/to/neonexus.db "node name"
cargo run -- --node-stop    /path/to/neonexus.db "node name"
cargo run -- --node-restart /path/to/neonexus.db "node name"
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
cargo test --test web
make web-smoke
cargo run -- --source-purity .
cargo run -- --source-quality .
cargo run -- --ci-policy .github/workflows/ci.yml
```

`cargo test --test web` boots real servers on ephemeral ports and exercises
the auth boundary, the JSON API, and the lifecycle control path end-to-end.

`make verify` runs the broader local gate set, including the web smoke, runtime
probes, alerts, readiness, metrics, integrity, support bundles, event
journals, node config export/generation, backups, wallets, launch packs, and
release-adjacent flows.

## Architecture

The source tree is intentionally Rust-only:

- `src/main.rs` is a thin binary entrypoint.
- `src/manager/` classifies startup arguments into the web workbench mode or
  explicit headless manager commands.
- `src/web/` is the browser workbench: axum router, auth store, page
  handlers, JSON API, and embedded assets. It renders server-side and calls
  only the core facade.
- `src/cli/` parses headless commands and renders text/JSON output.
- `src/core/` is the UI-free facade shared by the web workbench and CLI.
  High-level operations live here: `core::lifecycle` (node start/stop/restart
  orchestration), `core::node_health` and `core::workspace_queries` (read APIs
  so a surface never queries the repository directly during rendering).
- Domain modules such as `runtime`, `snapshots`, `config`, `launch`,
  `repository`, `backup`, `wallet`, `private_network`, `supervisor`,
  `source_purity`, `source_quality`, and `ci_policy` hold reusable behavior
  outside any surface.

Tests are kept out of `src/` so the source tree reads as production only:

- `tests/unit/` mirrors the `src/` module layout and holds the in-crate unit
  tests. Each production module keeps a one-line `#[cfg(test)] #[path = ...]
  mod tests;` stub that points at its `tests/unit/` file, so the tests retain
  private access while their code lives outside `src/`.
- `tests/web.rs` is the named end-to-end web target.
- `tests/domain`, `tests/ci_policy`, and `tests/repository` hold public-API
  integration tests compiled as separate test crates.

- `--source-purity` rejects Node/Web manifests, frontend source files,
  `node_modules`, web/frontend directories, Docker/compose and nginx
  deployment artifacts, WebView/Tauri project files, and WebView/Tauri
  dependencies. Browser assets live inside Rust string constants in
  `src/web/assets.rs` for exactly this reason.
- `--source-quality` rejects panic-oriented production markers, hardcoded
  platform shortcut labels, and oversized repository maintenance files.
- `--ci-policy` verifies cross-platform CI coverage on Ubuntu, macOS, and
  Windows with the Rust-only gate set and no frontend toolchain.

## Documentation

- [Web workbench](docs/web.md) explains the server, the auth model, cloud
  deployment, and the API surface.
- [Native Rust App Validation](docs/native-validation.md) records the gates
  and release evidence expected before handoff.
- [Operator Benchmarks](docs/operator-benchmarks.md) summarizes the node
  manager product patterns used to shape the workbench.
- [Runtime catalog example](docs/runtime-catalog.example.json) and
  [snapshot catalog example](docs/snapshot-catalog.example.json) are importable
  schema samples for Runtime Manager and Fast Sync workflows.

## Current Gaps

- More Linux and Windows smoke runs against real neo-cli, neo-go, and neo-rs
  binaries through the web workbench.
- More long-running process-supervision tests with real node data directories.
- Signed catalog and release-distribution exercises with real operator keys.
- Optional TLS termination in-process (today: put a reverse proxy in front).
