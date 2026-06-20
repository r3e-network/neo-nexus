# Native Rust App Validation

Validation date: 2026-06-20

## Scope

This report validates the pure Rust application in this repository. It checks
build correctness, tested behavior, native-application boundaries, and basic
engineering quality.

## Gates

```bash
cargo fmt --all --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- --self-check
cargo run -- --runtime-smoke neo-rs /definitely/missing/neo-node
cargo run -- --runtime-smoke-json neo-rs /path/to/neo-node
cargo run -- --rpc-health 127.0.0.1:1
cargo run -- --rpc-health-json 127.0.0.1:1 # expected exit code 1 for unreachable
cargo run -- --workspace-readiness /path/to/neonexus.db
cargo run -- --workspace-readiness-json /path/to/neonexus.db
cargo run -- --workspace-metrics /path/to/neonexus.db
cargo run -- --workspace-metrics-json /path/to/neonexus.db
cargo run -- --workspace-metrics-prometheus /path/to/neonexus.db
cargo run -- --workspace-integrity /path/to/neonexus.db
cargo run -- --workspace-integrity-json /path/to/neonexus.db
cargo run -- --source-purity /path/to/neo-nexus
cargo run -- --source-purity-json /path/to/neo-nexus
cargo run -- --source-quality /path/to/neo-nexus/src
cargo run -- --source-quality-json /path/to/neo-nexus/src
cargo run -- --source-quality /path/to/neo-nexus/tests
cargo run -- --source-quality-json /path/to/neo-nexus/tests
cargo run -- --native-ui-audit /path/to/neo-nexus
cargo run -- --native-ui-audit-json /path/to/neo-nexus
cargo run -- --ci-policy /path/to/neo-nexus/.github/workflows/ci.yml
cargo run -- --ci-policy-json /path/to/neo-nexus/.github/workflows/ci.yml
cargo run -- --alert-preview datadog "https://event-management-intake.datadoghq.com/api/v2/events?api_key=<DD_API_KEY>" critical "RPC health unreachable"
cargo run -- --alert-preview-json datadog "https://event-management-intake.datadoghq.com/api/v2/events?api_key=<DD_API_KEY>" critical "RPC health unreachable"
cargo run -- --export-readiness-report /path/to/neonexus.db /path/to/reports
cargo run -- --export-support-bundle /path/to/neonexus.db /path/to/support
cargo run -- --export-support-bundle-json /path/to/neonexus.db /path/to/support-json
cargo run -- --export-event-journal /path/to/neonexus.db /path/to/events
cargo run -- --export-event-journal /path/to/neonexus.db /path/to/events 100 warning restart
cargo run -- --export-node-configs /path/to/neonexus.db /path/to/configs
cargo run -- --export-node-configs-json /path/to/neonexus.db /path/to/configs
cargo run -- --generate-node-config neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --generate-node-config-json neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --validate-node-config neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --validate-node-config-json neo-rs testnet rocksdb 10332 10333 /path/to/config.toml
cargo run -- --export-backup /path/to/neonexus.db /path/to/backups
cargo run -- --export-backup-json /path/to/neonexus.db /path/to/backups
cargo run -- --import-backup /path/to/neonexus.db /path/to/neonexus-backup.json
cargo run -- --import-backup-json /path/to/neonexus.db /path/to/neonexus-backup.json
cargo run -- --validate-backup /path/to/neonexus-backup.json
cargo run -- --validate-backup-json /path/to/neonexus-backup.json
cargo run -- --validate-wallet /path/to/validator.wallet.json
cargo run -- --validate-wallet-json /path/to/validator.wallet.json
cargo run -- --import-wallet-profile /path/to/neonexus.db /path/to/validator.wallet.json validator-wallet "Validator wallet"
cargo run -- --validate-launch-pack /path/to/private-network/manifest.json
cargo run -- --launch-pack-sidecars /path/to/private-network/manifest.json
cargo run -- --launch-pack-sidecars-json /path/to/private-network/manifest.json
cargo build --release
target/release/neo-nexus --package-release dist
target/release/neo-nexus --verify-release-package dist
target/release/neo-nexus --verify-release-package-json dist
```

Expected result:

- Rust formatting passes.
- Rust debug build passes.
- Rust strict clippy passes with warnings denied.
- Rust tests pass.
- Headless binary self-check passes without opening a GUI window.
- Headless runtime smoke CLI reports a bounded blocked result without opening a
  GUI window, and its JSON mode emits structured preflight/attempt evidence
  with non-zero exit codes for blocked or failed probes.
- Headless RPC health CLI reports a bounded unreachable result without opening
  a GUI window, and its JSON mode emits method-level health evidence with a
  non-zero exit code for degraded or unreachable endpoints.
- Headless source purity gates reject Node/Web frontend manifests, frontend
  source files, `node_modules`, web/frontend directories, Docker/compose
  artifacts, nginx web-server deployment files, WebView/Tauri project files,
  and WebView/Tauri Cargo packages while skipping generated `target`, `dist`,
  and `.git` directories.
- Headless source quality gates reject panic-oriented Rust development markers
  plus document-style native layout markers such as scroll areas and virtual
  table builders in production source, allow assertion shortcuts in test
  sources, and reject Rust source files over 200 lines while skipping
  generated `target`, `dist`, and `.git` directories.
- Headless native UI audit gates require the desktop shell to use `eframe` and
  `egui`, start through the native eframe runner, set minimum window sizing,
  expose fixed top/bottom/left/right/central panels, define explicit workspace
  tabs, and reject WebView/Tauri/Wry or document-style scrolling markers.
- Headless CI policy gates require Ubuntu, macOS, and Windows workflow
  coverage, source purity/source quality checks, neo-rs runtime and generated
  config checks, release packaging verification, and no frontend, Node, Tauri,
  or WebView workflow tooling.
- Headless alert preview gates validate provider-specific targets and payload
  construction without sending a network request, then emit redacted text/JSON
  endpoint, header, and payload evidence for operators and CI.
- Native Alerts preview validates the draft route with the same request builder
  and renders redacted endpoint/header/payload evidence in the fixed policy
  panel without sending a webhook, including a current/stale marker when the
  draft changes after preview.
- Native workspace interaction stays fixed-panel and paginated, with no
  application scroll containers; cross-platform keyboard accelerators cover
  reload, node creation, selected-node lifecycle toggling, selected-node
  restart, numbered primary
  workspace selection, next/previous workspace cycling, and node inventory
  movement across rows, pages, first item, and last item.
- Workspace readiness and launch readiness report managed-config posture,
  warning when neo-go or neo-rs runtime args preserve an existing config flag
  and therefore bypass generated managed config injection.
- Launch readiness blocks configured active-node port collisions and occupied
  IPv4/IPv6 localhost TCP listeners before the native supervisor starts a node
  process.
- Restart readiness keeps the same active-node conflict protection but permits
  the selected running node's own local listeners before replacement.
- Headless workspace integrity CLI runs read-only SQLite `integrity_check`,
  foreign-key, required schema, required index, row-count, and page metadata
  checks, and its JSON mode exits non-zero when integrity fails.
- Headless workspace metrics CLI captures system resource pressure, process
  counts, managed node process CPU/memory/uptime, and missing running-node PIDs
  in terminal, structured JSON, and Prometheus text exposition forms.
- Native Operations workspace exposes the same read-only integrity check in
  the fixed workspace safety panel, stores the latest result for display, and
  writes a `workspace-integrity-checked` audit event.
- Native Operations event journal export writes filtered timestamped redacted
  text and JSON audit evidence and records an `event-journal-exported` event.
  The same export is available headlessly without opening a GUI window,
  including optional limit, severity, and query filters.
- Native Operations event journal rows are selectable and expose full
  structured event detail in the fixed panel, so operators can audit id, kind,
  node, timestamp, severity, and full message without relying on truncated row
  summaries.
- Native Operations and headless support bundle export write a diagnostics
  directory and ZIP archive containing readiness, read-only integrity, bounded
  event journal, metrics text/JSON/Prometheus snapshots, redacted node
  inventory, redacted runtime log diagnosis, privacy note, and SHA-256 manifest
  evidence, then record a
  `support-bundle-exported` audit event from the desktop action. The bundle is
  not a backup and excludes raw databases, raw runtime logs, private keys,
  wallet passwords, passphrases, mnemonics, seeds, authorization bearer
  values, API keys, tokens, webhook secrets, cached runtime packages, and
  snapshots.
- Headless bulk node config export writes neo-cli JSON, neo-go YAML, and
  neo-rs TOML files under collision-safe per-node directories, plus
  timestamped text/JSON evidence without opening a GUI window.
- Headless standalone node config generation writes neo-cli JSON, neo-go YAML,
  or neo-rs TOML without a workspace database, then immediately emits text/JSON
  validation evidence for the generated file.
- Headless backup export writes a timestamped JSON backup without opening a GUI
  window, and its JSON mode emits structured export evidence for automation.
- Headless backup import validates the backup before restore, refuses target
  workspaces with active nodes, and its JSON mode emits structured import
  evidence for automation.
- Headless encrypted wallet validation checks NEP-6 wallet structure,
  Base58Check account addresses, encrypted NEP-2 account keys, scrypt
  parameters, address-to-contract-script hash consistency, extracted contract
  public keys, and plaintext
  private-key/password/mnemonic/seed/token markers; failed text or JSON
  validations exit non-zero for CI.
- Native Wallet Profiles workspace and headless wallet profile import validate
  encrypted NEP-6 wallets before persistence, store metadata only, keep private
  keys, passwords, and copied wallet bytes out of SQLite, apply selected
  profiles to Roles signer references, and record `neo-wallet-profile-*` audit
  events.
- Rust release build passes.
- Release packaging writes a platform ZIP, sidecar JSON manifest, and `.sha256`
  checksum without opening a GUI window, then verifies sidecar, checksum,
  archive manifest, ZIP contents, and packaged binary hash. Text and JSON
  verification modes are both exercised.

## Native Boundary Checks

The source tree should contain only the Rust application, Rust tests, Cargo
metadata, documentation, and CI configuration. No alternate application runtime
or browser-wrapper project files should be present.

`--source-purity` and `--source-purity-json` make this boundary executable in
local smoke tests and CI. The gate scans the current file system tree, not git
history, and fails if Node/Web entrypoints such as `package.json`,
`node_modules`, `web/`, `*.ts`, `*.tsx`, `*.js`, HTML, or CSS files reappear.
It also fails if old server deployment artifacts such as `Dockerfile`,
`docker-compose.yml`, `nginx-example.conf`, or `setup-nginx.sh` return, if
Tauri project files such as `src-tauri/` or `tauri.conf.json` return, or if
`Cargo.toml` / `Cargo.lock` introduce WebView/Tauri packages such as `tauri`,
`wry`, `web-view`, `webview2-com`, or WebKit bindings.

## Professionalism Checks

Production Rust source should not contain panic-oriented development markers,
native layout markers that reintroduce document-style scrolling, or oversized
source files that should be split into focused modules. The enforced module
budget is 200 lines for both `src` and `tests`. Test sources may use assertion
shortcuts such as `unwrap` / `expect`, but unfinished markers and oversized
files still fail the quality gate:

```bash
rg -n "unwrap\\(|expect\\(|panic!|todo!|unimplemented!|dbg!|ScrollArea::|TableBuilder::|show_rows\\(|vertical_scroll\\(|horizontal_scroll\\(" src
cargo run -- --source-quality src
cargo run -- --source-quality-json src
cargo run -- --source-quality tests
cargo run -- --source-quality-json tests
cargo run -- --native-ui-audit .
cargo run -- --native-ui-audit-json .
cargo run -- --ci-policy .github/workflows/ci.yml
cargo run -- --ci-policy-json .github/workflows/ci.yml
```

The expected result is no production `rg` matches, `source-quality: ok` for
both `src` and `tests`, `native-ui-audit: native`, and `ci-policy: native-ci`.

## Functionality Covered By Tests

Current Rust tests cover:

- CLI parsing for GUI default, version/help output, self-check, runtime-smoke,
  runtime-smoke-json, rpc-health, rpc-health-json, headless event journal
  export, support bundle text/JSON export, workspace integrity text/JSON gates,
  source purity text/JSON gates, source quality text/JSON gates, native UI
  audit text/JSON gates, CI policy JSON gate, alert route preview text/JSON,
  headless node config export, headless node config export JSON, headless node
  config generation, headless node config generation JSON, headless node config
  validation, headless node config validation JSON, headless backup export,
  headless backup export JSON, headless backup import, headless backup import
  JSON, headless encrypted wallet validation, headless encrypted wallet
  validation JSON, headless encrypted wallet profile import, and invalid option
  rejection.
- Source quality detection for production panic-oriented Rust markers,
  document-style native layout markers that would reintroduce scroll/table
  work surfaces, test-aware assertion shortcuts, and oversized Rust files
  above the 200-line professional module budget.
- Native UI audit detection for required eframe/egui desktop shell markers,
  fixed application panels, explicit workspace tabs, minimum native window
  sizing, and forbidden WebView/Tauri/Wry or scrolling UI markers.
- Source purity detection for container and web-server deployment artifacts
  that would reintroduce the old server-oriented delivery model.
- CI policy detection for missing Linux/macOS/Windows matrix entries, missing
  native verification commands, weak neo-rs runtime-smoke gates without
  passed/hash/binary-evidence assertions, and forbidden frontend/Node/WebView
  workflow tooling.
- Runtime smoke binary path/size/SHA-256 evidence plus redaction for sensitive
  argv, stdout, stderr, and success messages while preserving raw spawn
  arguments internally.
- Launch-plan display command redaction, with raw process args preserved for
  actual node startup.
- Native application shortcut mapping for fixed workspace cycling, numbered
  primary workspace selection, selected-node restart, and filter-aware clamped
  node inventory navigation.
- Node Inventory filtering for status, name, runtime, network, storage,
  version, binary path, and port fields.
- Native Wallet Profiles application action for adding the selected profile's
  contract public key and encrypted wallet path to private-network signer
  references without duplicating existing references.
- Wallet Profile registry filtering for usage state, label, id, address,
  public key, wallet hash, source path, wallet version, and account counts.
- SQLite node creation, listing, definition updates, deletion, and status
  updates.
- Read-only workspace integrity reports for healthy SQLite databases, remote
  federation probe history schema coverage, and non-zero JSON failure evidence
  for foreign-key violations.
- Blank node name rejection.
- Plugin catalog content and state/category/query-filtered native registry.
- Per-node plugin enablement persistence.
- neo-cli plugin ZIP package installation with SHA-256 verification, safe
  extraction, replacement staging, manifest writing, non-neo-cli rejection,
  checksum mismatch rejection, and unsafe ZIP path rejection.
- SQL-backed plugin installation inventory persistence and node-delete cleanup.
- Role planner presets for neo-cli plugin state alignment and runtime-managed
  neo-rs posture.
- Runtime package SHA-256 verification, current-platform matching, local
  installation manifest writing, executable permission handling, and checksum
  mismatch rejection.
- Runtime binary preflight for missing binaries, host executable checks,
  neo-go binary recognition, neo-cli `dotnet Neo.CLI.dll` wrapper recognition,
  neo-rs `neo-node` naming, and non-blocking warnings for unexpected command
  names.
- Managed-config readiness posture for neo-go and neo-rs, including warning
  evidence when explicit runtime config arguments preserve an operator-supplied
  config instead of injecting NeoNexus generated config.
- Launch readiness port blockers for active NeoNexus node conflicts and
  occupied 127.0.0.1 or ::1 TCP listeners.
- Restart readiness for running nodes, including no false blocker from the
  node's own 127.0.0.1 or ::1 listeners and continued blocking for other
  active nodes.
- Runtime smoke probes for pass, review, blocked, and timed-out command
  outcomes with bounded process execution, runtime binary hash evidence,
  captured output summaries, and machine-readable JSON evidence.
- JSON-RPC health probes for healthy, degraded, and unreachable endpoints,
  covering `getversion`, `getblockcount`, JSON parsing, and error handling.
- SQL-backed RPC health record persistence, latest-record lookup, bounded
  per-node history reads, per-node pruning, and node-delete cleanup.
- Automatic RPC health event deduplication so repeated same-status background
  checks update history without flooding the runtime event journal.
- Persisted RPC health monitor policy settings for enablement and bounded
  automatic probe intervals.
- Alert routing validation for HTTPS/localhost provider targets, disabled
  default posture, stable Generic/Slack/Discord/Telegram/PagerDuty/Opsgenie/
  Datadog Events JSON payload fields, Telegram `chat_id` validation, PagerDuty
  Events API v2 `routing_key` validation, Opsgenie Alert API v2 `api_key`
  validation with `GenieKey` delivery headers, Datadog Events intake v2
  `api_key` validation with `DD-API-KEY` delivery headers, redacted native and
  text/JSON route previews, threshold matching, machine-local route settings,
  status/query-filtered delivery history, alert policy audit events, and
  delivery history persistence/pruning.
- Runtime package detached Ed25519 signature verification, invalid-signature
  rejection, and partial signature configuration rejection.
- Runtime HTTPS download request validation, HTTPS-only redirect policy,
  hash-checked local cache publishing, checksum mismatch rejection, and
  download size limit enforcement.
- Runtime release catalog JSON parsing, signed local catalog verification,
  tamper rejection, remote catalog signature requirement, host-compatible
  release selection, latest-version ordering, default download limits, manifest
  generation, runtime/platform/query-filtered native registries for releases
  and installed runtimes, and insecure URL rejection.
- Runtime trusted signer profile validation for reusable Ed25519 public keys.
- Runtime upgrade policy validation for catalog-profile requirements, safe
  interval bounds, due scheduling, per-run batch limits, UTC maintenance
  windows including overnight windows, and rollout wave delays.
- Runtime upgrade planning that prefers the latest matching installed package
  for a node's runtime and host platform.
- Runtime catalog upgrade planning that prefers the latest matching compatible
  release and skips current or older releases.
- Runtime catalog fleet upgrade planning that separates ready stopped nodes,
  blocked running nodes, and current/unavailable nodes.
- Native Runtime Manager selected-node catalog upgrades for running nodes,
  including pre-change restart readiness, runtime binary/version application,
  supervised process replacement, new PID persistence, `runtime-applied` audit
  evidence, and `node-restarted` audit evidence.
- SQL-backed runtime installation inventory persistence.
- SQL-backed runtime catalog source profile persistence, last-load metadata
  updates, and deletion.
- SQL-backed runtime signer profile persistence, last-used metadata updates,
  and deletion.
- SQL-backed runtime upgrade policy persistence, signed-catalog requirement
  settings, UTC maintenance windows, rollout wave delays, last-check metadata,
  and last-apply metadata.
- Private network topology planner output for deterministic neo-rs lab layouts.
- Private network plan materialization into node definitions without copying
  stale runtime arguments.
- Private network name/port conflict detection.
- Private network runtime profile rendering for neo-cli JSON, neo-go YAML, and
  neo-rs TOML.
- Private network launch pack export with managed config writing, manifest
  generation, per-node config SHA-256 inventory, generated artifact SHA-256
  inventory, start-order generation, deterministic seed list, validator count,
  operator-supplied committee public key injection, compressed public key
  validation, duplicate key rejection, signer wallet/HTTP(S) endpoint reference
  mapping, signer sidecar command-template expansion for `{wallet}`,
  `{endpoint}`, `{label}`, and `{public_key}`, unsafe signer endpoint
  rejection, native signer handoff summary counts for required signers,
  wallets, endpoints, sidecars, no-shell argv plans, supervisor-ready sidecar
  process specs, manifest reload of those specs after export, native Roles
  workspace start/stop/audit/watchdog restart and HTTP endpoint health checks
  for recovered signer sidecars, and text/JSON `--launch-pack-sidecars` CLI reporting for automation handoff, redacted
  sidecar text command evidence, schema v10 wallet provisioning summary
  generation, no-shell signer argv execution plan rendering, local/PATH
  sidecar binary validation, `wallet-provisioning.json` checklist rendering,
  `wallets/README.md` no-secret handling instructions, generated Unix/macOS
  and Windows preflight/start/health/stop
  scripts, binary/config/workdir/PID/port preflight checks, native-platform
  managed config SHA-256 preflight checks, signer wallet checks, encrypted
  NEP-6/NEP-2 signer wallet format validation, signer wallet address/script-hash
  consistency, signer wallet contract public key binding to the committee key,
  externally managed signer endpoint reachability checks, sidecar template and expanded
  command validation, default preflight and health invocation from start
  scripts, sidecar start before node start, signer PID/endpoint health probe
  rendering, post-start node
  PID/port/JSON-RPC health probe rendering, executable script permissions and
  shell syntax validation on Unix, reverse-order node and signer stop script
  rendering, and neo-rs consensus config activation for validator nodes.
- Private network launch pack offline validation from the native CLI, including
  schema checks, generated script presence, node binary/config/workdir checks,
  generated artifact SHA-256 integrity checks, managed config SHA-256
  integrity checks, duplicate port detection, signer public key validation,
  signer reference count validation, wallet provisioning policy/count/JSON
  consistency checks, wallet provisioning secret-boundary scanning for
  password, private-key, mnemonic, seed, token, and inline sensitive argv
  markers, native-platform signer wallet checks, encrypted wallet rejection when
  the file is missing NEP-6/NEP-2 structure, has an account address that does not
  match its contract script hash, carries plaintext key material, or does not
  bind to the manifest committee public key, endpoint URL validation,
  signer sidecar template validation, expanded command
  validation, argv execution plan consistency validation, local/PATH sidecar
  binary availability checks, missing endpoint warnings, and non-zero CLI exit
  status when validation fails.
- Native Roles workspace launch pack validation feedback after export,
  readiness summary retention in the fixed-panel UI, first-issue surfacing,
  same-pack revalidation after operator fixes, and validation audit events with
  severity derived from pass/warning/failure counts.
- Native Roles signer sidecar execution policy for recovered launch-pack
  sidecars, with bundled-only starts by default, an explicit `Allow External`
  operator override for PATH or pack-external signer binaries, workspace
  persistence/backup of that policy, watchdog restart enforcement of the same
  policy, and audited policy changes plus blocked starts.
- Launch pack validation report artifacts written as text and JSON inside the
  exported pack, refreshed after application export or same-pack revalidation.
- Launch pack handoff runbook generation with platform commands, validation
  report locations, signer material boundary, committee references, and node
  start order.
- Transactional multi-node creation with per-node role plugin state.
- Fast Sync local snapshot SHA-256 verification.
- Fast Sync cache writes that reject checksum mismatches before publishing a
  cached file.
- Fast Sync HTTPS snapshot download request validation, HTTPS-only redirect
  policy, size-limited hash-checked cache publishing, and checksum mismatch
  rejection before publish.
- Fast Sync snapshot catalog JSON parsing, signed local catalog verification,
  tamper rejection, HTTPS catalog signature requirement, catalog entry
  filtering, runtime/network/query-filtered native catalog and registry views,
  verification/cache state filters, and catalog-to-manifest conversion.
- Fast Sync application into a stopped node's managed data directory with an
  application manifest.
- Fast Sync runtime import for raw files, neo-rs `.tar.gz` packages, and `.zip`
  packages, including expanded byte accounting, imported-file counts, unsafe
  archive path rejection, and existing-target conflict rejection.
- SQL-backed Fast Sync snapshot manifest, HTTPS source metadata,
  verification, and cache metadata.
- Dashboard summary counts.
- Fleet diagnostics for port conflicts, runtime binary preflight, generated
  config validation, launch readiness, node readiness, local listener launch
  blockers, and headless workspace readiness CLI exit status, including
  machine-readable JSON output and timestamped text/JSON readiness report
  exports for CI and operator evidence.
- Neo-CLI plugin alignment diagnostics for RPC and storage plugins.
- neo-rs runtime parsing, built-in RPC diagnostics, and plugin-free catalog
  behavior.
- Workspace backup snapshots for node definitions, plugin state, plugin
  installation inventory, remote federation profiles, allowlisted workspace
  settings, runtime catalog profiles, trusted runtime signer profiles,
  metadata-only Neo wallet profiles, Fast Sync manifests, and recent runtime
  events.
- Workspace backup JSON reading, latest-backup discovery, safe import into
  stopped node definitions, plugin state and installation inventory
  restoration, workspace setting restoration, remote federation profile
  restoration, runtime profile restoration, signer restoration, snapshot
  manifest restoration without cached local file restoration, redacted runtime
  event history export and restoration with idempotent de-duplication, and
  headless backup export/import plus dry-run backup validation before import,
  including structured JSON output for CI.
- Backup dry-run validation rejects duplicate node IDs, duplicate node ports,
  duplicate plugin inventory, duplicate remote federation profile IDs,
  duplicate profile IDs, unsupported workspace setting keys, and invalid event
  kind/severity values before any SQLite write.
- Workspace backup import covers plugin state, unsupported setting rejection,
  and idempotent updates.
- Structured runtime event journal insertion, ordering, severity/kind parsing,
  role-application/private-network/config-application/runtime-installation
  event parsing, and deleted-node snapshot retention.
- SQL-backed runtime event filtering, counting, and bounded retention pruning.
- Native runtime event selection alignment for filtered event journal results,
  including automatic selection recovery when the previous selected event is no
  longer visible.
- Headless event journal report export with redacted text and JSON evidence,
  bounded limits, severity/query filters recorded in report metadata,
  matched/exported event counts, and latest-event ordering.
- Shared diagnostics redaction for event-journal messages, sensitive node
  arguments, and runtime log diagnosis summaries.
- Support bundle export with native action and text/JSON CLI evidence,
  redacted sensitive node arguments, redacted runtime log diagnosis summaries,
  privacy notes, manifest file hashes, directory output, ZIP archive output,
  and audit-event recording.
- neo-cli JSON, neo-go YAML, and neo-rs TOML config preview generation plus
  generated-config validation for parse correctness, network magic, storage,
  ports, seed lists, and neo-rs consensus posture.
- Headless single-file config validation for neo-cli JSON, neo-go YAML, and
  neo-rs TOML, including non-zero distinct expected port validation, ready JSON
  evidence, and critical drift exit codes.
- Headless single-file config generation for neo-cli JSON, neo-go YAML, and
  neo-rs TOML, including generated file path, byte count, text evidence, JSON
  evidence, and immediate semantic validation.
- Safe local JSON/YAML/TOML config export paths and exported file contents.
- Workspace-level config export reports, JSON stdout evidence, plugin counts,
  and duplicate-name-safe per-node output directories.
- Runtime-compatible storage validation that rejects neo-go RocksDB and
  prevents neo-rs LevelDB in newly saved node definitions.
- Managed config writes to exact runtime-specific paths, including neo-cli
  workdir `config.json` and neo-go / neo-rs files under `nodes/<id>/config/`.
- Managed config Apply + Restart uses the native process supervisor to replace
  a running child process, records the new PID, and preserves launch logging.
- Managed selected-node runtime catalog upgrade uses restart readiness and the
  native process supervisor to replace a running child process after applying
  the compatible installed runtime.
- General selected-node Restart uses the same native process supervisor,
  rewrites managed config when NeoNexus owns it, records `node-restarted`, and
  leaves stopped nodes on the Start path.
- Launch planning for neo-cli workdir config publication, neo-go managed
  `--config-file` injection, neo-rs managed `--config` injection, per-node
  working directories, and existing config argument preservation.
- Stable per-node process log paths and stdout/stderr log redirection on Unix.
- Generic managed-process supervisor coverage for non-node sidecars, including
  process kind logging, working-directory creation, stdout/stderr capture, and
  abnormal exit-code reaping, plus redacted command evidence in supervisor log
  headers.
- Managed-process stop coverage for graceful SIGTERM exits, bounded force-kill
  fallback after the grace period, and structured stop evidence in the process
  log.
- Bounded log snapshots, missing-log handling, and log clear behavior.
- Case-insensitive log search with preserved line numbers.
- Runtime log diagnosis for port binding failures, config parse errors,
  permission failures, missing files, database locks, runtime crashes, quiet
  logs, deduplicated findings, and actionable recommendations.
- Support bundle redaction for sensitive node argv shapes and runtime log
  excerpts, including key/value assignments, colon assignments, authorization
  bearer chains, passphrases, mnemonics, seeds, API keys, and tokens.
- Supervisor restart behavior after short-lived child processes exit.
- Supervisor finished-process reaping with exit codes.
- Repository cleanup for stale running/starting state from previous sessions.
- Native Monitor missing-process reconciliation that marks absent running PIDs
  stopped and records `runtime-state-reconciled` audit events.
- Port planning tests cover draft auto-assignment, selected-node repair,
  existing-node conflict avoidance, occupied IPv4/IPv6 localhost TCP
  avoidance, optional WebSocket allocation, and `node-ports-assigned` audit
  records.
- Watchdog restart scheduling, exponential backoff caps, due restart delivery,
  exhausted state, and manual clear behavior.
- Disabled watchdog behavior, runtime policy updates, pending restart clearing,
  and persisted watchdog policy settings.
- Resource telemetry formatting, pressure classification, node process totals,
  missing-process snapshot modeling, telemetry refresh, and missing-process
  repair.
- Shared argv parsing/display formatting, Node draft argument conversion,
  quoted runtime args, unterminated-quote rejection, loaded-args display
  formatting, and explicit per-node non-zero/distinct port validation.
- Pagination bounds.
- Fixed-panel text clipping helpers.
- Remote federation URL normalization, public status parsing, SQLite profile
  CRUD, status/query profile filtering, automatic monitor policy persistence,
  retained probe history persistence/pruning, status/query probe history
  filtering, native create/update/toggle/delete actions, disabled-profile probe
  recording, status-change event throttling, and audit event recording.

## Cross-Platform CI

GitHub Actions runs the Rust verification gates on Ubuntu, macOS, and Windows.
The matrix installs Linux GUI build dependencies, runs format/check/clippy/test,
runs `cargo run -- --self-check`, validates source purity, source quality,
native UI audit, and CI policy gates, validates text and JSON runtime/RPC health CLI paths, runs
workspace readiness, workspace integrity, support bundle, event journal, node
config, standalone neo-rs generated config, encrypted wallet validation, and
backup/export/import CLI paths,
builds the release binary, executes the release binary's `--self-check` path
on each platform, packages the native binary with `--package-release`, verifies
it with `--verify-release-package` and `--verify-release-package-json`, and
uploads the ZIP/manifest/checksum trio as workflow artifacts. The CI policy
gate rejects future workflow edits that drop a required OS, omit a native gate,
or reintroduce frontend/Node/Tauri/WebView tooling.
- Native Settings release package actions are covered by unit tests that create
  a package, verify the resulting manifest/ZIP/checksum, and assert
  `release-packaged` and `release-package-verified` audit events.

## Remaining Product Gaps

- Runtime-specific in-process live reload without restart where runtimes
  support it.
- Audited encrypted Neo wallet generation and richer wallet-authoring flows,
  plus deeper signer protocol checks beyond process execution policy and HTTP
  reachability.
- Additional provider integrations beyond the current alert routes and
  headless metrics exports, such as logging, uptime, and broader
  incident-management services.
- Linux and Windows smoke tests against real neo-cli / neo-go / neo-rs
  binaries.
