# Changelog

All notable changes to NeoNexus are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **A refused snapshot manifest comes back with what you typed.** Registering a
  fast-sync archive is nine fields — an id, a label, a network, a runtime, a
  source path, an URL, a download file name, a size limit and a 64-character
  digest — and a rejected save used to answer with a redirect, so one wrong
  character in the digest cost all nine. The page now comes back with every field
  as posted and the complaint beside the one that caused it; the fields whose
  answer this console can be sure of are attributed one by one, and a manifest
  the workspace refuses for a reason that belongs to no single field is quoted
  above the form in its own words.
- **A refused signer policy comes back with what you typed.** The key page's
  policy boundary is twenty-nine fields wide, and it used to answer a rejected save
  with a redirect and a flash line — leaving the operator to retype a boundary
  they had just copied from a design document. A boundary this console cannot
  read is now refused *before* the request is sent, and the page comes back with
  the posted text still in the form and the sentence beside the field that
  caused it: a rolling window with a ceiling and no length names the length, not
  the ceiling. A refusal from the custody service itself keeps its own words and
  is shown above the form, because it belongs to no single field. The one case
  that still redirects is a page that cannot be drawn at all — if the service
  stops answering between the save and the redraw, the redirect says so instead
  of pretending the form came back.
- **`GET /ready` — a readiness probe an orchestrator can act on.** `/healthz`
  answers "is this process alive"; `/ready` answers "should traffic come
  here". It is public, like `/healthz` and `/api/public/status`, and it
  answers `503` while the workspace is blocked or cannot be judged, so a load
  balancer or a Kubernetes service stops sending traffic to an instance that
  cannot serve it, and starts again once the operator has cleared what was
  blocking. Which node is blocked, and where its configuration lives, stay
  behind the operator session and `/api/readiness`.
- **`GET /api/jobs` — the background job list as JSON.** The pages already
  render long installs, but a deployment pipeline waiting for a runtime to
  land needs the list in a form it can poll. Every job carries its lane, its
  status, the result text, and — while it runs — a progress report with the
  stage, the completed and total counts and the percentage.
- **A federation server can be edited again.** The route that updates a peer
  existed from the start, but nothing on the page pointed at it — a peer could
  be added and never changed. The list now links to an edit form that arrives
  pre-filled, and a rejected save comes back on the same page with what you
  typed still in place and the reason beside the field, instead of a redirect
  to a blank form.
- **Form fields now carry the constraints the domain already enforces.**
  Ports, byte counts, hashes, and names are bounded in the browser as well as
  on the server. The bounds follow what each field *means*: a field that is
  allowed to be empty (leaving a port blank asks the workspace to pick one)
  gets a range but is not marked required, and an amount is still typed as
  text — `"1e6"` has to reach the custody service so that *it* can refuse it
  in its own words, which a numeric input would silently prevent.
- **Empty lists now say what to do next.** Nineteen pages showed a plain note
  where nothing had been configured yet, and a list with no way forward is a
  dead end rather than a state. Each one now carries the action that would
  fill it.
- **A failed form keeps what you typed.** The runtime upgrade policy form
  returned a redirect on failure, so a single out-of-range value threw away
  nine fields of work. It now redraws the form with the error next to the
  field that caused it and the rest of the input intact.
- **Long operations say they are running.** Exporting a support bundle,
  writing a readiness report, checking integrity, and moving a backup all take
  long enough that the page looks frozen. Each now shows a spinner when it is
  submitted. The form still submits if the script cannot run — the marker is
  a hint, never a gate.

- **Two more regression tests for that rule, and a fixed injection case.**
  The rule reads a literal after its escapes are resolved, so
  `<button onchange=\"go()\">` is caught — but nothing proved it, because
  the injection case that should have covered it was written with quotes that
  did not survive the trip from the harness into Rust and failed to compile.
  The harness was right to report it: a compile failure is not a rule
  firing. The payload now uses a raw string, and two resident tests cover
  the raw-string and the escaped-quote forms of a `format!` argument.
- **A third CI contract rule: no inline event handlers.** The console serves
  `script-src 'sha256-<hash of the bundled script>'` with no `'unsafe-inline'`,
  so a browser never compiles an `onclick` or an `onchange` — the handler is
  skipped and the control does nothing. Three of them had shipped. One guarded
  the deletion of every log file in the workspace: the confirmation never
  appeared and the form posted straight through. The other two left the
  JSON-RPC toggle and the role-preset cards inert, with nothing on the page to
  say why. The rule reads the syntax tree, so a comment that names an
  attribute and a sentence of prose about one are not offences; only markup
  carrying an actual event attribute is.
- **`GET /logs/clear` — a confirmation page before the logs are cleared.**
  Clearing the logs truncates every file in the workspace log directory, for
  every instance. The button that started it carried its confirmation in an
  inline handler the content security policy blocks, so the operator saw no
  prompt at all. The confirmation is now a page of its own, reached by a GET
  and completed by the POST that follows: it works with scripting disabled,
  and it states what is about to be destroyed.
- **Two CI contract rules that actually hold.** `tests/ci_policy/contract.rs`
  now reads the syntax tree instead of the text, so a rule can no longer pass
  because it failed to look at the code:
  - *Platform-specific executables.* A literal handed to `Command::new` is
    rejected unless a compile-time `#[cfg]` gate keeps it away from every
    platform where the executable does not exist. `#[cfg(unix)]` around
    `Command::new("sh")` is allowed; the same call without a gate is not. The
    gate is evaluated against the CI matrix itself — ubuntu, macos and
    windows, the last with two toolchains — and a gate that says nothing
    about platforms, a feature flag for instance, never narrows the scope.
  - *Documented endpoints.* Every endpoint `docs/openapi.yaml` and
    `docs/AGENT_API.md` promise must be routed, with the method it promises.
    The comparison reads the router's syntax tree, so a comment between the
    path and the handler no longer hides a route, and a quoted string that is
    not a path no longer invents one.
- **Revoking an access key is a page, not a prompt.** The table's Revoke button
  relied on `data-confirm`, which is behaviour of the bundled script: with
  scripting disabled the key was revoked on the first click, with nothing in
  between to read. `GET /settings/api-tokens/{token_id}/delete` now names the
  key, its access key id, when it was created, when it expires, and exactly
  which grants are about to be withdrawn, and says that every program
  presenting the key starts receiving `401` immediately and that the key cannot
  be reissued — the secret is stored only as a hash, so a replacement is a new
  credential each caller has to be given.
- **Removing a federation peer is a page, not a row button.** `Remove` posted
  straight from the table and took the peer's stored probe history with it.
  `GET /federation/{id}/delete` now states that the history goes too, that the
  peer's own deployment is untouched, that readding it means entering its name
  and base URL again, and whether the peer is currently polled. The list links
  to it instead of posting; both pages are plain forms, so the destructive step
  is never one click away and works with scripting disabled.

### Fixed

- **A runtime catalogue that was refused threw away six fields.** The catalogue
  form carries an identifier, a label, a source, an optional signature source, an
  optional ed25519 key and a size limit — and it sits at the bottom of a page
  whose top half is the inventory you were reading. A rejected save answered with
  a redirect, so a mistyped number cost everything above it. The form now comes
  back as posted, with the reason beside the size limit when that is what is
  wrong, and — when the refusal is a sentence about *two* controls, such as a
  signature source without its key — the domain's own words above the form
  rather than a field count that would name a control which is not wrong.
- **A restore that could not read the archive threw the path away.** The restore
  panel takes one value — the path to an archive on this host — and it answered a
  bad one with a redirect, so an operator pasted a long path, read one sentence,
  and pasted it again. The path now stays in the field with the reason beside it,
  and the page says the workspace is untouched. Drawing the page again turns out
  to be another read of the workspace; when that fails too, the redirect happens
  and says both things rather than landing on a blank form with no explanation.
- **A wallet that would not validate threw away all three fields.** The import
  form carries a path, an optional id and an optional label, and the path is the
  one that takes longest to type — and the one the failure is usually about. All
  three now come back as typed, with the reason beside the path. A profile that
  validated but could not be stored is a different failure: it belongs to no
  single field, so it is quoted above the form instead.
- **A settings save that failed said nothing on the page.** The restart budget
  and the two interval monitors answered a rejected save with a redirect, so the
  page came back with a flash line and an empty form — and when the failure
  belonged to the policy as a whole rather than to one field, the message was
  dropped entirely: the structure holding field errors was handed to the field
  controls only, and a control recognises its own name and nothing else, so the
  one key that belongs to no field was skipped in silence. All three forms now
  come back with what you typed, the reason beside the field that caused it, and
  — when no single field is at fault — the reason itself above the form. The
  count of failing fields and that whole-form sentence are now one shared rule
  instead of three implementations that disagreed.
- **A failure and a success looked the same.** Every control reported its
  outcome in a banner of one colour, so the only way to tell a failed start
  from a successful one was to read the sentence. The severity now travels
  with the message: a failure is rendered in red and announced assertively, a
  success in green and announced politely. Fifty-six failure paths across the
  console now say which they are.
- **Long operations held a worker thread.** Exporting a support bundle,
  writing a backup archive, clearing the logs and hashing a workspace all ran
  straight inside an `async fn`, pinning a Tokio worker thread for the whole
  operation — while the server serves every other request from a fixed pool of
  those threads. They move to the blocking pool now, as the node lifecycle
  controls already did.
- **Creating an API key could land on a blank page.** Submitting the form with
  no permission selected answered `400` with a bare sentence: the navigation,
  the page and everything typed into the form were gone. The form comes back
  with the problem stated on it and the name still in the field. The page that
  shows the new secret is a full page too, so the copy button on it works.
- **Disabling something looked like doing something.** Turning a plugin off, a
  federation peer off, a signing key off or a caller off used the ordinary
  control button. Turning them back on still uses the ordinary button, because
  putting a capability back is not the act that can go wrong.
- **The CI contract rules were green without looking.** The platform rule
  only recognised raw string literals, so an ordinary
  `Command::new("/bin/sleep")` passed the gate unnoticed; the endpoint rule
  compared paths without their methods and matched every quoted string that
  began with `/`, which both hid real drift and reported routes that were
  never missing. Both rules now read the syntax tree, and each was verified
  to fail on an injected violation and to pass on the legitimate forms.
- **A missing CI workflow step no longer aborts the test process.**
  `tests/ci_policy/workflow_shells.rs` turned a missing step into a `panic!`,
  which the source-quality gate forbids and which killed the whole test
  binary instead of failing one assertion. The step is now looked up as a
  recoverable error, and the test fails with the step name in the message.
- **Test ports no longer compete with the operating system.** The suites
  hardcoded their ports at 33332, 43332, 52332 and a dozen others — inside the
  range the operating system hands out for temporary source ports (49152 and up
  on Windows, 32768 and up on Linux). Any unrelated outbound connection could
  take one of those ports, and the readiness preflight then refused to start
  the node with "RPC port 52332 is already listening" — a failure that has
  nothing to do with the code under test, and one that struck the web suite
  six runs out of six while a proxy held the port. Every test port now lives
  below 32768, which is outside the ephemeral range of every platform the
  matrix covers.
- **A log-collection test no longer depends on a thread race.**
  `engine_log_collection_bounds_reads_and_tolerates_missing_logs` wrote the
  log file after the collector had started, but never woke it: the collector
  samples first and parks for 30 seconds afterwards, so the event the test
  waited for arrived only on the next round — and only if the write had
  happened before the collector's first sample. Under load the collector won
  the race and the test failed roughly one run in six. It now wakes the
  collector explicitly, the way its sibling test does.
- **Opening a workspace no longer fights a running server.** `Repository::open`
  re-ran the full schema bootstrap on every call, including the write-locking
  `journal_mode` pragma and DDL. A second handle — a CLI inspection command, or
  a test harness — could therefore fail intermittently with `attempt to write a
  readonly database` while the web server was writing. The schema version is now
  recorded in `PRAGMA user_version`, and a current workspace answers one
  read-only query and returns.
- **The workspace journal mode is `TRUNCATE`, not WAL.** WAL removes its `-shm`
  file when the last connection closes; with a connection per operation that
  transition intermittently produced `SQLITE_READONLY` for the next writer on
  Windows. The workload is a management console, so the reader/writer exclusion
  costs nothing that matters and the attach is deterministic.
- **Transient attach failures are retried.** `ReadOnly`, `Busy` and `Locked`
  while opening a connection are retried with backoff instead of surfacing as a
  failed write, and the connection is forced to attach its journal before the
  caller starts an operation.
- **Platform-neutral runtime fixtures.** The failed-launch, watchdog-crash and
  restart tests now use compiled stand-in binaries instead of `sleep`, `/bin/sh`,
  `cmd.exe` or `timeout`. The previous Windows "late crash" fixture exited
  inside the launch settle window (`timeout` refuses redirected input), so the
  watchdog path was never exercised and the test only passed by accident.
- **`ci_policy` workflow parsing is line-ending agnostic.** The gate passed on
  Linux and failed on a Windows checkout with `core.autocrlf=true` because step
  markers were matched against LF only.

### Erratum for 1c4114c

Commit 1c4114c ("mount the integration suite, and stop calling plaintext
encrypted") overstated what it did, and it left CI red.

- It said 68 previously invisible tests were now running. Of those 68, 18 made
  real assertions about the product. The other 50 asserted nothing about it:
  they ran against a test-local `NodeManager` with a fixed PID 12345 (the real
  `neo_nexus::node_manager::NodeManager` never existed), exercised fixtures,
  helpers and a mock metrics server, only printed "coverage", or used asserts
  such as `result.is_ok() || result.is_err()` that cannot fail. Those 50 are
  gone. The 18 real ones are kept in `tests/integration/node_types.rs` or moved
  to library unit tests.
- It said the repository has no crypto dependency. `Cargo.toml` lists aes,
  scrypt, p256, ed25519-dalek and sha2.
- It failed CI's `Format` step and had 6 Clippy errors, all in code it added.
- The binary's own `--help` still called the backup encrypted.

`cargo test --test integration` now runs 17 tests, all through the public
`neo_nexus` API with no test-local stand-ins for product code. Nine drive the
lifecycle core shared by the web workbench and
`--node-start`/`--node-stop`/`--node-restart` (`execute_node_launch`,
`stop_node_runtime`) over a real SQLite workspace, with the real
`LaunchPlanner`, `ConfigExporter` and `ProcessSupervisor` spawning a real child
process. For each of neo-cli, neo-go, neo-rs, neox-geth and neox-rs, a start
writes a parseable managed config at the path the launch command names, records
`Running` with the pid of a live process and logs the launch; a stop ends that
process, records `Stopped` with no pid and logs the stop. Restart replaces the
process. All five run side by side under one supervisor while a node with a
missing runtime fails alone in `Error`. The other eight check node-id and port
validation, node-type round-trips, chain family, default storage engine,
plugin support and config-path extensions. The runtime is a compiled stand-in
that ignores its arguments and sleeps, so no real Neo client, sync, RPC,
readiness check, signer duty or plugin is exercised.


### Security

- **Backup is described as what it is everywhere**: `--help` now reads "Write a
  0600 plaintext JSON workspace export, NOT encrypted", and the documented-claims
  gate reads the rendered `--help` and whole statements rather than single lines,
  so reworded, wrapped or ambiguous encryption claims about the backup fail it.
  `docs/MIGRATION-v4.3.md` no longer tells operators to run a
  `make backup-export FORMAT=tar.gz` target that does not exist.
- **Dependency advisory cleanup**: upgrade `rustls` to 0.23.45 (RUSTSEC-2026-0285,
  TLS 1.3 handshake message boundary) and `anyhow` to 1.0.104 (unsound
  `Error::downcast_mut`). Both are now absent from `cargo audit`.
- **Dependency gate in CI**: add a RustSec advisory sweep (`cargo audit`) and a
  `cargo deny` license/bans/sources check (`deny.toml`) to the verification
  pipeline, and to `make verify`.

### Changed

- **Close the test-coverage gap in CI**: `tests/integration.rs`,
  `tests/ui_density_metrics.rs`, and `tests/ui_operator_walkthrough.rs` now run
  in the CI `Test` step and in `make test`/`make verify`, instead of being left
  to smoke only.

### Fixed

- **UI density walkthrough**: the comfortable-mode assertion now checks the
  node's genuine independent axes (`Process`, `Chain health`, `Height`) rather
  than the retired fused "Status Check" column, matching `tests/web.rs`.
- **4th parity gate**: every HTTP surface declared in `docs/AGENT_API.md`
  (as `### METHOD /path`) must be served by the router, with path parameters
  normalised so `{id}` and `{node_id}` match. The three earlier gates pin each
  surface the code names to its pair; this one anchors the automation protocol
  reference to the implementation, so a declared route the router does not
  serve fails the build. Verified by injecting a bogus endpoint.

### Docs

- Mark the root `NODE_MANAGER_*`, `PHASE*`, and `BENCHMARKS_STATUS.md`
  files as historical snapshots superseded by the custody/observation
  refactor, pointing to `claudedocs/NEONEXUS_GAP_REGISTER.md` as the live
  TODO/gap register.

### Refactor

- Split `src/web/node_form.rs` (760 lines) into `src/web/node_form.rs`
  (the `NodeDraft` model and its constructors/queries) plus
  `src/web/node_form/validate.rs` (the port/name/version rules that turn a
  parsed draft into a `NewNode`). The rules are a self-contained, purely
  internal concern and now read beside nothing else; the two modules keep
  their behaviour under the existing 26 node-form unit tests.
- Split `src/web/pages/nodes/iac_spec.rs` (703 lines) into
  `src/web/pages/nodes/iac_spec.rs` (the `IacFormat` dispatch and the in-page
  card) plus `src/web/pages/nodes/iac_spec/formats.rs` (the per-target IaC
  string builders). The format renderers are pure functions that produce
  Docker, K8s, CloudFormation, Terraform, CLI and JSON text; they now live
  apart from the table that dispatches to them. Behaviour is unchanged under
  the two existing iac_spec unit tests.

## [v4.3.1] - 2026-09-10

### Fixed

- **Node lifecycle event audit**: manual node launch failures in web/CLI paths are now recorded as `RuntimeLaunchedFailed` events, ensuring the Event Journal captures when an operator or automation initiates a Start that cannot complete due to workspace locks, binary mismatches, or port conflicts.

## [4.3.0] - 2026-09-10

This release expands the web workbench to reach actions that were previously
CLI/API-only, adds API token authentication for CI/automation, and resolves
findings from an Ultra CodeReview pass.

### Added

- **API token authentication for CI/automation** (P1-5): SHA-256-hashed bearer
  tokens with a `TokenPermission` enum, a two-layer `require_session` +
  `require_permission` middleware, and an `AuthIdentity` that distinguishes
  `Session` from `Token` callers, so automation can authenticate without a
  browser session while inheriting least-privilege permission scoping.
- **Prometheus metrics independent authentication route** (P0-1):
  `/api/metrics-prometheus` is now reachable with the `ReadFleet` permission
  rather than only a browser session, so an external scraper no longer has to
  authenticate as a browser does.
- **Web wallet profile import** (P0-2): `POST /wallets/import` with SHA-256
  validation and path-traversal protection.
- **Web wallet profile delete** (P2-3): `POST /wallets/{id}/delete`, recording a
  `NeoWalletProfileDeleted` event.
- **Web snapshot lifecycle events** (P1-1): save/verify/download/cache handlers
  now record `SnapshotSaved`, `SnapshotVerified`, `SnapshotDownloaded`, and
  `SnapshotCached` events.
- **Web plugin package installation** (P1-2): multipart upload processed on a
  background job with a 2GiB cap and SHA-256 verification, journaling a
  `PluginInstalled` event.
- **Web runtime smoke test trigger** (P1-3): `POST /nodes/{id}/smoke-test`
  records a `RuntimeSmokeTested` event.
- **Web log clear functionality** (P1-4): operators can clear a node's logs from
  the browser.
- **Runtime upgrade policy edit form** (P1-7): `POST /settings/runtime-upgrade`
  edits the policy from Settings, recording a `RuntimeUpgradePolicyUpdated`
  event.
- **`workspace_root` environment variable configuration** (P1-6): the workspace
  directory can now be set from the environment.
- **`ConfigApplied` and `NeoWalletProfileUsed` event producers** (P2-7).
- **Scheduled runtime upgrade execution** in the supervision engine: `LoopState::probe_runtime_upgrade()` periodically evaluates fleet-wide upgrade opportunities via `RuntimePackageManager::plan_catalog_fleet_upgrades()`, applies updates within configurable intervals and maintenance windows, automatically stopping running nodes before installation and restarting them post-upgrade. Policy configuration (`enabled`, `interval_minutes`, `max_nodes_per_run`, `maintenance_window`) takes effect without restarts; events are journaled to the Event Journal.

- **Web UI for Snapshot Apply**: POST handler `/snapshots/{snapshot_id}/apply/{node_id}` that verifies snapshot readiness, validates compatibility (network type, node type), calls `FastSyncSnapshotManager::apply_to_node()` to apply snapshots to compatible nodes, and returns flash-message feedback. Snapshots page now shows Apply buttons for verified, cached snapshots alongside compatible nodes.

- **Web UI for Backup Export**: POST handler `/backup/export` that triggers workspace backup exports using `WorkspaceBackupExporter`, displays a new `/backup` page with export status indicators and stat tiles (nodes, signer profiles, snapshots, events count), and records `BackupExported` audit events with artifact summary messages.

- **Private network magic override security fix**: Cross-replay prevention mechanism combining node identity binding, one-time token consumption, and monotonic generation counters with TTL enforcement. Magic override tokens are rejected when attempted on wrong nodes (`WrongNode`), expired (`Expired`), or replayed (`AlreadyConsumed`). Integrated into config export path to validate tokens before applying magic number changes.

- **60 unit tests for private_network module** covering scripts, verifier, exporter, validation support (wallets, sidecars), committee parsing, reports rendering, and signers endpoint handling — replacing placeholder assertions with behavioral coverage.

- **4 unit tests for supervision runtime upgrade scheduling** (`test_probe_runtime_upgrade_disabled_by_default`, `test_probe_runtime_upgrade_respects_interval`, `test_probe_runtime_upgrade_respects_maintenance_window`, `test_probe_runtime_upgrade_returns_early_without_catalog_config`) verifying policy gating logic without network dependencies.

### Changed

- **Runtime upgrade settings surface** changed from read-only facts to an
  editable form, paired with the `POST /settings/runtime-upgrade` handler.

### Fixed

- **Ultra CodeReview**: `apply_snapshot` now derives the workspace path from
  `data_dir` instead of the hardcoded `"workspaces"` directory.
- **Ultra CodeReview**: the plugin upload body limit uses `DefaultBodyLimit::max`
  as a 2GiB DoS safety net.
- **Ultra CodeReview**: failed runtime smoke-test events are raised to
  `Critical` severity for operator visibility.

## [4.2.0] — 2026-09-08

**Note:** This release was initially planned and documented; implementation completed in v4.1.x development cycle.

### Added

- Compact single-line inventory and fleet `node_row` anatomy (40pt slots)
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

## [4.1.0] — 2026-09-06

### Added

- A named signer registry that can load `local-wallet`, loopback-HTTPS
  `local-signer`, and HTTPS `neo-os-service` profiles concurrently. Console,
  public-relay, and default internal-signing routes are selected independently;
  key identity is backend-qualified, selection never relies on URL guessing,
  and runtime failure never falls back across profiles.
- A process-local encrypted NEP-6 wallet signer for Neo N3/P-256. It pins and
  revalidates wallet identity, reads the passphrase only from a protected file,
  strictly parses the complete unsigned transaction, enables transaction
  signing by default, refuses consensus without durable anti-equivocation,
  keeps raw signing explicit and default-off, and is never exposed through the
  public relay or remote administration surfaces. The decrypted scalar is
  zeroizing but remains cached until the profile's last clone drops.
- The Rust signer-service integration split into configuration, secret-file,
  authentication, transport, admin/signing clients, and wire-model modules.
  It supports protected admin token files, distinct least-privilege internal
  signing identities, canonical Ed25519 workload assertions, mandatory
  authenticated TLS, and transparent relay of caller proofs and exact body
  bytes. Unix permissions and Windows owner/DACL validation fail closed.
- A dedicated Security / Signer workspace for key inventory, policy, caller and
  audit operations, with confirmation pages for destructive or credential-
  rotating actions. Key import and private-key/passphrase inputs remain outside
  the web boundary.
- A searchable Events surface and a redesigned operations-console information
  architecture: Overview, Fleet, Operations, Network, Assets, Security, and
  Workspace.
- Hash-pinned Content Security Policy and global no-store, anti-framing,
  anti-sniffing, referrer, permissions, COOP/CORP, and conditional HSTS headers.
- Main-branch build provenance through GitHub artifact attestations, alongside
  the existing explicitly pinned Ed25519 publisher-verification path.

### Changed

- Node Start, Stop, Restart, watchdog recovery, CLI control, and browser control
  now share a CAS-backed lifecycle protocol. Restart first proves and quiesces
  the recorded process; stop intent is durable before termination; PID identity,
  exit confirmation, and concurrent edits/deletes fail closed.
- Backup-restored runtime commands are quarantined until an operator explicitly
  rebinds a trusted local binary. Release ZIPs enforce entry, size, duplicate,
  path, and streaming digest limits; webhook redirects are disabled.
- Browser authentication accepts strong protected token files only in service
  mode, rate-limits login attempts, enforces exact-origin writes, scrubs control-
  plane secrets from child processes, and emits Secure cookies for HTTPS origins.
- The public federation API now exposes aggregate counts only; node identity,
  version, health, host, and process inventories require an operator session.
- Forms, tables, focus behavior, skip navigation, filter controls, labels, live
  regions, and mobile navigation were reworked for keyboard and screen-reader
  use. Background jobs no longer force a focus-destroying page refresh.

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
  - Auth: single strong operator token from `--web-token-file` or
    `NEONEXUS_WEB_TOKEN_FILE` (interactive-only bootstrap on loopback; only its
    SHA-256 digest is retained), per-peer login throttling, exact-origin checks
    for protected writes, and an HttpOnly, SameSite=Strict session cookie with
    `Secure` on HTTPS deployments and 12-hour sliding expiry.
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
- **Lifecycle and control audit trail.** Sweeping which event kinds still had a
  producer turned up 52 of 65 with none, and the ones that mattered were actions
  the workbench itself offers: Start, Stop and Restart wrote no journal entry, so
  "who stopped this node at 03:00" had no answer; neither did the plugin toggle,
  the workspace config export, or saving any of the four policies. All of them
  are journaled now, carrying the node they refer to so an entry is
  attributable rather than merely present. The remaining unproduced kinds belong
  to capabilities that still have no frontend and are listed under Known gaps
  rather than being quietly left out.
- **Startup reconciliation** in the engine: nodes left recorded as Running by a
  previous instance are checked against the host before anything is concluded.
  One whose process is still alive keeps its status — a workbench killed with
  SIGKILL leaves its nodes running, and clearing those rows would lose the only
  handle on them, so the next Start would launch a second node onto the same
  ports. One whose process is gone is settled and journaled as
  `RuntimeRecovered`. A pid answered by a *different* program is settled too but
  reported separately, because the number was recycled and the old process is
  not coming back. `Repository::clear_transient_runtime_state`, which had no
  production caller after the desktop shell went away, is deliberately not used
  here: it clears every Running row unconditionally, which is correct for an app
  that kills its own children on exit and wrong for a server.
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
- **Readable time** (`src/web/time.rs`): every table that showed a raw Unix
  second now shows `2026-08-29 12:47Z` with the elapsed reading beneath it
  ("3m ago"), and a machine-readable `<time datetime>`. Fixed-width so a column
  aligns, UTC because that is what the logs, events and probes are written in.
  The conversion is days-since-epoch arithmetic rather than another dependency,
  and is tested against independently known anchors and the leap-year rollovers
  where hand-rolled date code usually breaks.
- **Layout and interaction polish**: wide tables scroll instead of squeezing
  eight columns into slivers; a page with a job in flight refreshes itself when
  it finishes and stays put when idle; the flash and job panels announce through
  `aria-live`; the active nav item carries `aria-current`; headings keep their
  spacing whether or not they sit in a page header; federation probe history
  gained the breadcrumb the rest of the sub-pages use.
- **Background jobs** (`src/web/jobs.rs`): the install runs on its own thread
  behind a one-job-per-lane registry, so a multi-minute download cannot time out
  a browser, a reload still shows it running, and two concurrent installs cannot
  interleave writes into the same tree. The page reports state, result and
  failure reason.
- `--web` / `--bind` / `--port` / `--web-token-file` /
  `--web-public-origin` launch options. No options starts the web workbench (the
  default interactive experience).
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
- **Several desktop capabilities still have no frontend at all**, discoverable
  as event kinds with no producer: role application (`RoleApplied`), plugin
  package install (`PluginInstalled`), the snapshot lifecycle (`SnapshotSaved`,
  `SnapshotVerified`, `SnapshotDownloaded`, `SnapshotCached`, `SnapshotApplied`),
  wallet profile import and deletion, private-network materialisation with its
  signer sidecars, backup export/import/validation, log clearing, and runtime
  smoke tests. Each is implemented in the core layer; none is reachable.
- **Scheduled runtime upgrades are not enforced.** The policy is stored,
  validated and displayed, and `RuntimePackageManager::plan_node_upgrade`,
  `plan_catalog_upgrade` and `plan_catalog_fleet_upgrades` all exist — but
  nothing calls them, so no node is ever upgraded on an interval. The Settings
  page now says so rather than implying the policy is live. The scheduler in
  `src/app/runtime_upgrade_policy/execution.rs` went away with the desktop
  shell and has not been replaced.
- `/api/metrics-prometheus` sits behind the session cookie, so an external
  Prometheus scraper must authenticate as a browser does or scrape through an
  internal route.
- TLS is not terminated in the binary; a reverse proxy is expected in front of
  the bound address.

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

[4.3.0]: https://github.com/r3e-network/neo-nexus/compare/v4.2.0...v4.3.0
[4.2.0]: https://github.com/r3e-network/neo-nexus/compare/v4.1.0...v4.2.0
[4.0.0]: https://github.com/r3e-network/neo-nexus/compare/v3.1.0...v4.0.0
[3.1.0]: https://github.com/r3e-network/neo-nexus/compare/v3.0.0...v3.1.0
