# Operator Benchmarks

NeoNexus is shaped as a professional node-operations application. This document
captures the product patterns borrowed from mature node management tools and
how they are applied in the pure Rust native workbench.

## Reference Products

- Dappnode organizes node packages around operational panels such as Info,
  Config, Network, Logs, File Manager, and Backup:
  <https://docs.dappnode.io/docs/user/packages/understanding-dappnode-packages/overview/>
- Stereum emphasizes guided setup, a dashboard, monitoring tools, and flexible
  node configuration:
  <https://stereum-dev.github.io/ethereum-node-web-docs/>
- Umbrel's Bitcoin Node highlights explicit version choice, advanced network
  settings, and compatibility guidance:
  <https://apps.umbrel.com/app/bitcoin>
- Rocket Pool Smartnode frames node operation as a lifecycle of secure,
  maintain, monitor, and upgrade:
  <https://docs.rocketpool.net/node-staking/cli-intro>
- ethereum.org describes mature launchers as tools that automate setup and
  provide guided setup, monitoring, and control:
  <https://ethereum.org/developers/docs/nodes-and-clients/run-a-node/>
- neo-rs is the Rust Neo N3 node implementation with a `neo-node` daemon and
  TOML configuration:
  <https://github.com/r3e-network/neo-rs>

## Applied Product Principles

- Workbench over pages: operators should not move through a website-like flow
  to maintain nodes. NeoNexus keeps inventory, workspace, inspector, command
  controls, and status visible as a fixed desktop layout.
- Guided remediation: a warning should identify the target workspace, action,
  and reason. Readiness findings include stable keys, severity, target
  workspaces, action labels, and hints.
- Bounded context: long lists use filters and paging. Node selection, severity
  filters, query filters, and target workspace filters persist as operators
  move between fleet and selected-node triage.
- Native command discoverability: menus, toolbar buttons, and keyboard
  accelerators share one command model for reload, node creation, lifecycle
  controls, workspace selection, and inventory movement.
- Evidence-first operations: health checks, readiness, event history,
  integrity, metrics, backups, support bundles, wallet validation, launch-pack
  validation, and release verification all emit operator-readable and
  automation-readable evidence.

## Runtime And Upgrade Patterns

- Runtime provenance: packages are installed only after platform matching,
  SHA-256 verification, HTTPS-only download posture, size limits, atomic cache
  publication, and optional detached Ed25519 signature validation.
- Runtime parity: neo-cli, neo-go, and neo-rs are treated as production
  runtimes, each with native config generation, launch planning, smoke probes,
  readiness findings, and evidence export.
- neo-rs parity: `neo-node` recognition, RocksDB posture, TOML config
  generation, managed `--config` injection, runtime catalog entries, Fast Sync
  snapshots, and private-network validator configs are first-class flows.
- Upgrade safety: stopped-node upgrade application and running-node catalog
  upgrade restarts use readiness checks, batch limits, maintenance windows,
  rollout delays, and event-journal summaries.

## Monitoring And Triage Patterns

- RPC health is collected in bounded background probes and stored with recent
  trend history.
- Resource monitoring exposes host pressure, managed-process CPU/RSS/uptime,
  and missing PID state without leaving the native application.
- Logs are captured to stable files, diagnosed for common startup problems,
  searched in fixed panels, and linked to abnormal-exit notices.
- Event history is structured, filterable, selectable for full detail, and
  exportable as text/JSON evidence.
- Port conflicts are surfaced as actionable rows that can focus the relevant
  node and workspace.

## Safety Patterns

- Source purity gates prevent the project from drifting back into a frontend,
  WebView, Tauri, Node, or server-container app.
- Native UI audits prevent scroll-first document pages from replacing the
  desktop workbench.
- Source quality gates keep Rust modules, docs, JSON examples, and maintenance
  files reviewable.
- Wallet workflows store encrypted wallet metadata only and reject plaintext
  secret markers.
- Support bundles are diagnostics, not backups, and deliberately exclude raw
  databases, raw logs, wallet secrets, tokens, webhook secrets, packages, and
  snapshot caches.
- Backup restore refuses workspaces with active nodes and restores definitions
  as stopped.

## Private-Network Patterns

Mature node tools make topology and signer setup explicit. NeoNexus applies
that principle by exporting private-network launch packs with deterministic
runtime configs, validator counts, seed lists, network magic, committee public
keys, wallet references, signer endpoints, sidecar command templates, no-shell
argv plans, runbooks, validation reports, and secret-boundary reminders.

The application does not generate validator private keys or silently embed
secrets. Operators provide encrypted wallets and signer endpoints; NeoNexus
validates references and rejects inline secret material.

## Release Handoff Patterns

Settings and CLI release flows both package the native executable, write a
manifest and checksum, verify the archive, and record audit evidence. This
mirrors node-operations products where release artifacts must be inspected and
handed off without relying on a running GUI session.

## Current Boundaries

NeoNexus now has a native application shell, dual GUI/headless manager,
runtime catalog and Fast Sync systems, process supervisor, managed configs,
runtime upgrades, readiness triage, logs, metrics, backups, support bundles,
wallet profiles, private-network launch packs, release packaging, source
purity, source quality, native UI, and CI policy gates.

The next product-hardening work should focus on real-binary cross-platform
smoke runs, long-running node supervision, real signed catalogs, accessibility
review, and deeper keyboard-only operator walkthroughs.
