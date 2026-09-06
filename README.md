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

**Signing and key custody.** NeoNexus can load named profiles for all three
backend families in one process: process-local encrypted NEP-6 wallets,
locally deployed signers, and NeoOS signer services. A
`NEONEXUS_SIGNER_PROFILES_FILE` TOML registry explicitly selects the console,
public-relay, and default internal-signing profiles; every key reference contains
both `backend_id` and `key_id`, and a failed target never falls back. Legacy
`NEONEXUS_SIGNER_BACKEND` remains a single-profile compatibility path. Local
wallets are Neo N3/P-256 transaction signers only: the unsigned transaction is
strictly parsed, consensus is refused without durable anti-equivocation, raw is
default-off, and the wallet is never attached to the public relay. Service
profiles keep admin and least-privilege signing credentials separate and use
authenticated TLS, including for loopback production deployments. The Signer
page exposes remote administration only for the explicitly selected service
profile and never exposes key import. See [the registry example](docs/signer-profiles.example.toml),
`D:\Git\neo-os\neo-os-services\docs\SIGNER_SERVICE.md` for the contract and
ownership matrix, and `docs/signer-service-audit.md` for the findings that
constrain it.

## Requirements

- Rust 1.91 or newer.
- Linux, macOS, or Windows.
- Optional node binaries if you want to start real processes:
  `neo-cli`, `neo-go`, or neo-rs `neo-node`.

No GUI, WebView, Node.js, or frontend-toolchain dependency is required to
build NeoNexus. Platform CI compiles and tests the same Rust-only source tree
on Linux, macOS, and Windows.

## Run The Web Workbench

```bash
cargo run
```

No options starts the workbench server on `127.0.0.1:8080`. In an interactive
terminal it prints the address plus a generated sign-in token. Redirected or
service startup fails closed unless a protected token file is configured, so a
bootstrap credential cannot leak into logs.

Cloud-shaped options:

```bash
umask 077
openssl rand -hex 32 > /run/secrets/neonexus-web-token
cargo run -- --web --bind 0.0.0.0 --port 8080 \
  --web-token-file /run/secrets/neonexus-web-token \
  --web-public-origin https://nexus.example.com
```

- `--bind` defaults to `127.0.0.1`; set `0.0.0.0` on a cloud host behind a
  TLS-terminating reverse proxy.
- `--web-token-file` or `NEONEXUS_WEB_TOKEN_FILE` names a regular, access-
  controlled file containing a token of at least 32 bytes. Secret-bearing
  `--web-token` and `NEONEXUS_WEB_TOKEN` inputs are refused. Only the SHA-256
  digest is retained after startup.
- A non-loopback listener also requires `--web-public-origin` (or
  `NEONEXUS_WEB_PUBLIC_ORIGIN`) with the browser-facing HTTPS origin.
- Sessions use 12-hour, HttpOnly, SameSite=Strict cookies; HTTPS deployments
  also set `Secure`. Protected state-changing requests must present the exact
  configured Origin (or a same-origin Referer when Origin is absent).

The workspace database lives at `NEONEXUS_DATA_DIR/neonexus.db` (or the OS data
directory), the same file the CLI writes to — browser operators and scripted
operators see the same workspace.

## Workbench Surfaces

| Area | Pages | What operators do there |
|------|-------|--------------------------|
| **Overview** | Fleet overview | See fleet posture, host pressure, attention items, and live node state |
| **Fleet** | Nodes, Health, Logs | Register and control nodes, inspect process/RPC health, and diagnose logs |
| **Operations** | Readiness, Events, Alerts | Resolve launch blockers, search the event journal, and route actionable alerts |
| **Network** | Federation, Private network | Observe peer workspaces and plan role-aware private networks |
| **Assets** | Runtimes, Snapshots, Plugins, Configuration, Wallets | Manage verified runtime inputs and metadata without accepting wallet secrets |
| **Security** | Signer | Inspect all loaded local-wallet, local-signer, and NeoOS service profiles plus their explicit route roles; administer keys, policies, callers, and audit only through the selected console service profile |
| **Workspace** | Metrics, Settings | Export metrics and tune watchdog, RPC, and federation policies |

Status badges poll `/api/fleet` every 5 seconds; every control also works
without JavaScript (plain form posts with flash messages). `/healthz` and the
inventory-minimized `/api/public/status` are the only public status surfaces;
node identities and host/process detail remain authenticated.
`/api/metrics-prometheus` serves the same Prometheus exposition the CLI exports.
All responses are `no-store` and carry a strict hash-pinned CSP plus standard
frame, MIME, referrer, permissions, and cross-origin hardening headers.

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
cargo run -- --node-rebind-runtime /path/to/neonexus.db "imported node" /local/path/to/neo-node [runtime-args...]
```

Backup imports retain the supplied command for backup fidelity but quarantine
it from execution. The imported node has no active binary or arguments until
an operator explicitly selects a trusted local runtime with the rebind command
above (or saves a local runtime through the node editor).

After a release build:

```bash
cargo build --release
target/release/neo-nexus --package-release dist
target/release/neo-nexus --verify-release-package-integrity dist
target/release/neo-nexus --verify-release-package-integrity-json dist
```

Those two commands prove integrity only. A production handoff must also sign
the exact canonical `*.manifest.json` bytes with an externally held Ed25519
private key, publish the detached signature as base64 text, and authenticate it
with an independently distributed/pinned public key:

```bash
target/release/neo-nexus --verify-release-package dist "$NEONEXUS_RELEASE_PUBLIC_KEY_B64" dist/neo-nexus-<version>-<platform>.manifest.sig
```

NeoNexus intentionally ships no production trust anchor or signing key.
Main-branch CI packages also receive a GitHub build-provenance attestation and
the CI policy gate requires that step to remain present. Consumers can verify
that independent trust path with `gh attestation verify <artifact> -R <owner>/<repo>`;
it complements, but does not replace, an operator-pinned Ed25519 publisher key.

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
- Domain modules such as `runtime`, `snapshots`, `config`, `launch`, `signing`,
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
- [Signer service design](docs/signer-service-design.md) maps all three current
  signer backends, the v1 service client/control-plane contract, and the original
  custody-engine design retained as historical policy background.
- [neo-os signer audit](docs/signer-service-audit.md) records the workspace-wide
  findings those constraints answer to, each with file and line evidence.

## Current Gaps

- More Linux and Windows smoke runs against real neo-cli, neo-go, and neo-rs
  binaries through the web workbench.
- The remote-signer Neo CLI adapter is intentionally ABI-locked to official
  `neo-node` 3.9.2. A separately reviewed build is required for a newer Neo CLI
  plugin ABI; NeoNexus refuses a version mismatch instead of guessing.
- More long-running process-supervision tests with real node data directories.
- Signed catalog and release-distribution exercises with real operator keys.
- Optional TLS termination in-process (today: put a reverse proxy in front).
