# NeoNexus

NeoNexus is a pure Rust operations workbench for Neo N3 node fleets. One
binary starts a **web workbench**: open the printed address in a browser and
operate your node fleet from anywhere — a laptop, a bastion host, or a cloud
server. The same binary also exposes the full headless CLI for scripts and CI.

There is no desktop application since 4.0.0. The workbench runs as an HTTP
service (axum + tokio), renders server-side, and ships its browser assets
inside the binary — no Node toolchain, no external services, one executable.

## What Operators Can Do

- **Multi-Engine Node Fleet Management**: First-class support for Neo N3 (`neo-cli`,
  `neo-go`, `neo-rs` `neo-node`) and Neo X EVM (`neox-geth`, `neox-rs`) from the web
  workbench or headless CLI.
- **One-Click & Rolling Fleet Upgrades**: Canary progression, rolling update plans,
  automated pre/post-upgrade health verification, and zero-downtime rollback.
- **Configuration Conflict & Drift Reconciliation**: Port conflict planner across P2P,
  RPC, Prometheus, and sidecar endpoints; real-time configuration drift detection with
  automated timestamped backup and zero-loss reconciliation.
- **State Checkpoints & Fast-Sync Snapshots**: Snapshot catalog integration, validated
  backup archives (AES-GCM encrypted, tar.gz), and zero-trust quarantined imports.
- **Neo-CLI Plugin Lifecycle & Sidecars**: Plugin catalog dependency management,
  conflict prevention, and automated sidecar configuration injection (`ApplicationLogs`,
  `StateService`, `RpcServer`, `TokensTracker`).
- **P2P Topology, Mempool & Dual-Family RPC Probes**: Deep health probes for Neo N3 and Neo X
  including peer connectivity/isolation warnings, transaction pool congestion monitoring,
  on-chain role designations, and Prometheus metrics export.
- **Headless API Token Administration**: Role-based access control (`admin_all`,
  `read_fleet`, `read_readiness`) for automated CI/CD pipelines and headless operations.
- **Unified Process Supervision**: Launch, stop, restart, and inspect supervised node
  processes through the shared core pipeline (readiness → managed config → supervise → persist).

neo-rs is a first-class runtime target. NeoNexus recognizes the `neo-node`
binary, validates RocksDB-oriented TOML configs, supports Fast Sync snapshot
catalog entries, and routes neo-rs readiness findings into the same Operations
workflow used for neo-cli and neo-go. Neo X (`neox-geth` and `neox-rs`) provides
EVM-compatible execution, dual-family RPC probes, and multi-node orchestration.

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

## Headless CLI & Feature Verification

All operational commands run without the workbench and share its core pipeline. Every feature can be verified directly via the CLI with consistent exit codes and dual text/JSON formatting for automation:

### 1. Node Lifecycle & Process Supervision
Control and inspect supervised node processes across all 5 runtimes (`neo-cli`, `neo-go`, `neo-rs`, `neox-geth`, `neox-rs`):

```bash
# List all registered nodes in workspace (tabular or structured JSON)
cargo run -- --node-list /path/to/neonexus.db
cargo run -- --node-list-json /path/to/neonexus.db

# Supervise node status, launch, stop, and restart
cargo run -- --node-status /path/to/neonexus.db "node-01"
cargo run -- --node-status-json /path/to/neonexus.db "node-01"
cargo run -- --node-start /path/to/neonexus.db "node-01"
cargo run -- --node-stop /path/to/neonexus.db "node-01"
cargo run -- --node-restart /path/to/neonexus.db "node-01"

# Rebind node runtime binary (used after migration or quarantined backup restore)
cargo run -- --node-rebind-runtime /path/to/neonexus.db "node-01" /local/path/to/neo-node
```

### 2. Runtime Smoke & One-Click Upgrade Verification
Smoke check node engine candidates before executing one-click or rolling fleet upgrades:

```bash
# Smoke test candidate runtime binary for any engine (3-second execution probe)
cargo run -- --runtime-smoke neo-rs /path/to/neo-node
cargo run -- --runtime-smoke-json neo-rs /path/to/neo-node
cargo run -- --runtime-smoke neox-geth /path/to/geth
cargo run -- --runtime-smoke neo-cli /usr/bin/dotnet /path/to/neo-cli.dll

# Package and authenticate release distributions (integrity + Ed25519 signature)
cargo run -- --package-release dist
cargo run -- --verify-release-package-integrity dist
cargo run -- --verify-release-package-integrity-json dist
cargo run -- --verify-release-package dist "$NEONEXUS_RELEASE_PUBLIC_KEY_B64" dist/manifest.sig
```

### 3. Configuration Generation, Conflict & Drift Reconciliation
Generate golden node configs, detect port collisions (P2P/RPC/metrics), audit drift, and safely reconcile:

```bash
# Generate deterministic configuration (supports neo-cli, neo-go, neo-rs, neox-geth, neox-rs)
cargo run -- --generate-node-config neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --generate-node-config-json neo-rs testnet rocksdb 10332 10333 /path/to/config.toml

# Validate existing configuration file against network rules and storage engine schemas
cargo run -- --validate-node-config neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --validate-node-config-json neo-rs testnet rocksdb 10332 10333 /path/to/config.toml

# Detect configuration drift between disk file and workspace golden spec
cargo run -- --check-config-drift /path/to/neonexus.db "node-01" /path/to/config.toml
cargo run -- --check-config-drift-json /path/to/neonexus.db "node-01" /path/to/config.toml

# Reconcile drifted configuration (automatically creates timestamped backup before applying)
cargo run -- --reconcile-node-config /path/to/neonexus.db "node-01" /path/to/config.toml
cargo run -- --reconcile-node-config-json /path/to/neonexus.db "node-01" /path/to/config.toml

# Bulk export configs for all workspace nodes
cargo run -- --export-node-configs /path/to/neonexus.db /path/to/configs_dir

# Validate private network launch pack & sidecar configurations
cargo run -- --validate-launch-pack /path/to/private-network/manifest.json
cargo run -- --launch-pack-sidecars /path/to/private-network/manifest.json
```

### 4. State Checkpoints, Snapshots & Disaster Recovery
Manage state snapshots, export encrypted workspace backups, and restore with zero-trust execution quarantine:

```bash
# Export encrypted workspace backup archive (AES-GCM / tar.gz)
cargo run -- --export-backup /path/to/neonexus.db /path/to/backups
cargo run -- --export-backup-json /path/to/neonexus.db /path/to/backups

# Validate backup archive integrity, manifest, and database schema without applying
cargo run -- --validate-backup /path/to/backups/backup.tar.gz
cargo run -- --validate-backup-json /path/to/backups/backup.tar.gz

# Restore workspace backup into database (quarantines node runtimes until explicitly rebound)
cargo run -- --import-backup /path/to/target.db /path/to/backups/backup.tar.gz
cargo run -- --import-backup-json /path/to/target.db /path/to/backups/backup.tar.gz

# Export comprehensive support diagnostic bundle (logs, metrics, events, readiness)
cargo run -- --export-support-bundle /path/to/neonexus.db /path/to/support_dir
cargo run -- --export-support-bundle-json /path/to/neonexus.db /path/to/support_dir
```

Backup imports retain the supplied command for backup fidelity but quarantine
it from execution. The imported node has no active binary or arguments until
an operator explicitly selects a trusted local runtime with `--node-rebind-runtime`
(or saves a local runtime through the node editor).

### 5. Dual-Family RPC Probes, P2P Topology & Mempool Telemetry
Probe endpoint health, P2P connectivity, transaction congestion, and query on-chain roles across Neo N3 and Neo X:

```bash
# RPC health probe with latency, block height, and protocol checks (N3 & Neo X auto-detect)
cargo run -- --rpc-health 127.0.0.1:10332
cargo run -- --rpc-health-json 127.0.0.1:10332

# P2P peer connectivity probe (isolated / sparse / healthy detection)
cargo run -- --peer-health 127.0.0.1:10332 [neo-n3|neo-x]
cargo run -- --peer-health-json 127.0.0.1:10332 [neo-n3|neo-x]

# Mempool backlog and congestion status (normal / elevated / congested)
cargo run -- --mempool-status 127.0.0.1:10332 [neo-n3|neo-x]
cargo run -- --mempool-status-json 127.0.0.1:10332 [neo-n3|neo-x]

# On-chain role designation probe (state-validator, oracle, neofs-alphabet, p2p-notary)
cargo run -- --designation 127.0.0.1:10332 state-validator [public-key]
cargo run -- --designation-json 127.0.0.1:10332 state-validator [public-key]

# Governance committee snapshot query
cargo run -- --governance 127.0.0.1:10332
cargo run -- --governance-json 127.0.0.1:10332
```

### 6. Headless API Token Administration (RBAC)
Manage scoped API tokens for automated CI/CD pipelines, Prometheus scrapers, and external orchestrators:

```bash
# Create scoped API token (permissions: read_fleet, read_readiness, admin_all)
cargo run -- --create-api-token /path/to/neonexus.db "ci-deployer" admin_all
cargo run -- --create-api-token /path/to/neonexus.db "metrics-scraper" read_fleet

# List active API tokens
cargo run -- --list-api-tokens /path/to/neonexus.db
cargo run -- --list-api-tokens-json /path/to/neonexus.db

# Revoke token by UUID
cargo run -- --revoke-api-token /path/to/neonexus.db <TOKEN_UUID>
```

### 7. Diagnostics, Readiness & Event Journal
Evaluate fleet posture, search audit trails, and export telemetry:

```bash
# Evaluate overall workspace readiness (identifies port conflicts, missing runtimes, config flaws)
cargo run -- --workspace-readiness /path/to/neonexus.db
cargo run -- --workspace-readiness-json /path/to/neonexus.db
cargo run -- --export-readiness-report /path/to/neonexus.db /path/to/readiness-report.json

# Workspace database integrity check
cargo run -- --workspace-integrity /path/to/neonexus.db
cargo run -- --workspace-integrity-json /path/to/neonexus.db

# Export Prometheus metrics
cargo run -- --workspace-metrics-prometheus /path/to/neonexus.db
cargo run -- --workspace-metrics-json /path/to/neonexus.db

# Query and export event journal audit logs
cargo run -- --export-event-journal /path/to/neonexus.db /path/to/journal_export 100 all
cargo run -- --alert-preview /path/to/neonexus.db [provider] [webhook_url]

# Validate NEP-6 wallet metadata (strictly without accepting private keys)
cargo run -- --validate-wallet /path/to/wallet.json
cargo run -- --import-wallet-profile /path/to/neonexus.db /path/to/wallet.json
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

- [Web workbench](docs/web.md) explains the server, the auth model, cloud deployment, and the API surface.
- [Agent Protocol & Automation API](docs/AGENT_API.md) covers REST API endpoints, API token RBAC, and headless CLI JSON automation commands.
- [Troubleshooting & Operations Runbook](docs/TROUBLESHOOTING.md) provides operator runbooks for config drift reconciliation, P2P isolation, mempool congestion, port collisions, and disaster recovery.
- [Node Support & Lifecycle Verification](docs/NODE_SUPPORT_VERIFICATION_REPORT.md) records the comprehensive audit and verification for all 5 node engines, rolling upgrades, configuration conflict detection, checkpoint snapshots, and plugin management.
- [Plugin Support Matrix](docs/PLUGIN_SUPPORT_MATRIX.md) details plugin and extension capabilities across Neo N3 and Neo X runtimes (C# DLL ZIP installer, sidecar injection, and build-time features).
- [System Architecture & Security Audit](docs/SYSTEM_AUDIT_2026.md) records the full system evaluation, security posture, domain boundaries, and quality ratings.
- [Native Rust App Validation](docs/native-validation.md) records the gates and release evidence expected before handoff.
- [Operator Benchmarks](docs/operator-benchmarks.md) summarizes the node manager product patterns used to shape the workbench.
- [OpenAPI 3.0 Specification](docs/openapi.yaml) provides the machine-readable API specification for automated client generation.
- [Runtime catalog example](docs/runtime-catalog.example.json) and [snapshot catalog example](docs/snapshot-catalog.example.json) are importable schema samples for Runtime Manager and Fast Sync workflows.
- [Signer service design](docs/signer-service-design.md) maps all three current signer backends, the v1 service client/control-plane contract, and the original custody-engine design retained as historical policy background.
- [neo-os signer audit](docs/signer-service-audit.md) records the workspace-wide findings those constraints answer to, each with file and line evidence.

## Current Gaps

- More Linux and Windows smoke runs against real neo-cli, neo-go, and neo-rs
  binaries through the web workbench.
- The remote-signer Neo CLI adapter is intentionally ABI-locked to official
  `neo-node` 3.9.2. A separately reviewed build is required for a newer Neo CLI
  plugin ABI; NeoNexus refuses a version mismatch instead of guessing.
- More long-running process-supervision tests with real node data directories.
- Signed catalog and release-distribution exercises with real operator keys.
- Optional TLS termination in-process (today: put a reverse proxy in front).
