//! # NeoNexus - Multi-Node Supervisor & Autonomous Workstation for Neo & EVM
//!
//! NeoNexus provides full lifecycle supervision, local key custody, metrics exposition,
//! and automated watchdog self-healing for Neo N3, NeoGo, Neo-Rs, and NeoX networks.
//!
//! ## Subsystem Architecture
//! - **Node Engine**: Configuration export, process supervision, port planning, and lifecycle orchestration.
//! - **Custody & Signer**: Local encrypted wallet custody, EVM signing, caller policy, and HTTP relay.
//! - **Workspace & Data**: High-concurrency SQLite storage (WAL mode), CQRS read/write services, and backups.
//! - **Observability**: Prometheus metrics normalization, RPC health monitoring, and audit journals.
//! - **Federation & Network**: Private network genesis generation, role planning, and runtime upgrades.
//! - **Frontends & Adapters**: Authenticated Web console, headless CLI commands, and argument parsing.
//! - **Governance & Quality**: Continuous source quality, pure-rust purity, and CI policy enforcement.

// ── 1. Node Engine & Process Supervision ────────────────────────────────────
pub mod agents;
mod child_environment;
pub mod config;
pub mod launch;
pub mod node_lifecycle;
pub mod port_planner;
pub mod preflight;
pub mod runtime_smoke;
pub mod supervision;
pub mod supervisor;
pub mod watchdog;

// ── 2. Custody & Signer Relay ───────────────────────────────────────────────
pub mod signer_client;
pub mod signing;
pub mod wallet;

// ── 3. Workspace & Storage (CQRS) ───────────────────────────────────────────
pub mod backup;
pub mod core;
pub mod repository;
mod secret_file;
pub mod snapshots;
pub mod workspace_integrity;
mod workspace_lock;

// ── 4. Observability, Metrics & Diagnostics ─────────────────────────────────
pub mod alerts;
pub mod dashboard;
pub mod diagnostics;
pub mod event_journal_report;
pub mod events;
pub mod health_events;
pub mod logs;
pub mod metrics;
pub mod observe;
pub mod readiness_report;
pub mod redaction;
pub mod rpc_health;
pub mod support_bundle;

// ── 5. Federation, Roles & Network ──────────────────────────────────────────
pub mod catalog;
pub mod chain_state;
pub mod federation;
pub mod plugins;
pub mod private_network;
pub mod release_pack;
pub mod roles;
pub mod runtime;

// ── 6. Frontends & CLI ─────────────────────────────────────────────────────
pub mod argv;
pub mod cli;
pub mod manager;
pub mod web;

// ── 7. Quality, Utilities & Types ───────────────────────────────────────────
pub mod ci_policy;
pub mod source_purity;
pub mod source_quality;
pub mod types;
pub mod utils;
