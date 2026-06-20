use std::{env, fs, path::PathBuf, process, time::SystemTime};

use anyhow::{Context, Result};

use crate::repository::Repository;

pub(in crate::cli) fn version_text() -> String {
    format!("NeoNexus {}", env!("CARGO_PKG_VERSION"))
}

pub(in crate::cli) fn help_text() -> String {
    let mut text = format!(
        "{version}\n\nUSAGE:\n  neo-nexus [--version|--self-check|--help]\n  neo-nexus --runtime-smoke <neo-cli|neo-go|neo-rs> <binary> [runtime-args...]\n  neo-nexus --runtime-smoke-json <neo-cli|neo-go|neo-rs> <binary> [runtime-args...]\n  neo-nexus --rpc-health <port|url>\n  neo-nexus --rpc-health-json <port|url>\n  neo-nexus --workspace-readiness <neonexus.db>\n  neo-nexus --workspace-readiness-json <neonexus.db>\n  neo-nexus --workspace-metrics <neonexus.db>\n  neo-nexus --workspace-metrics-json <neonexus.db>\n  neo-nexus --workspace-metrics-prometheus <neonexus.db>\n  neo-nexus --workspace-integrity <neonexus.db>\n  neo-nexus --workspace-integrity-json <neonexus.db>\n  neo-nexus --source-purity <repo-dir>\n  neo-nexus --source-purity-json <repo-dir>\n  neo-nexus --source-quality <source-dir>\n  neo-nexus --source-quality-json <source-dir>\n  neo-nexus --ci-policy <workflow.yml>\n  neo-nexus --ci-policy-json <workflow.yml>\n  neo-nexus --export-readiness-report <neonexus.db> <output-dir>\n  neo-nexus --export-support-bundle <neonexus.db> <output-dir>\n  neo-nexus --export-support-bundle-json <neonexus.db> <output-dir>\n  neo-nexus --export-event-journal <neonexus.db> <output-dir> [limit] [info|warning|critical|all] [query...]\n  neo-nexus --export-node-configs <neonexus.db> <output-dir>\n  neo-nexus --export-node-configs-json <neonexus.db> <output-dir>\n  neo-nexus --validate-node-config <neo-cli|neo-go|neo-rs> <mainnet|testnet|private> <leveldb|rocksdb> <rpc-port> <p2p-port> <config-path>\n  neo-nexus --validate-node-config-json <neo-cli|neo-go|neo-rs> <mainnet|testnet|private> <leveldb|rocksdb> <rpc-port> <p2p-port> <config-path>\n  neo-nexus --export-backup <neonexus.db> <output-dir>\n  neo-nexus --export-backup-json <neonexus.db> <output-dir>\n  neo-nexus --import-backup <neonexus.db> <backup.json>\n  neo-nexus --import-backup-json <neonexus.db> <backup.json>\n  neo-nexus --validate-backup <backup.json>\n  neo-nexus --validate-backup-json <backup.json>\n  neo-nexus --validate-wallet <wallet.json>\n  neo-nexus --validate-wallet-json <wallet.json>\n  neo-nexus --validate-launch-pack <manifest.json|launch-pack-dir>\n  neo-nexus --launch-pack-sidecars <manifest.json|launch-pack-dir>\n  neo-nexus --launch-pack-sidecars-json <manifest.json|launch-pack-dir>\n  neo-nexus --package-release <output-dir>\n  neo-nexus --verify-release-package <dist-dir|manifest.json|archive.zip>\n  neo-nexus --verify-release-package-json <dist-dir|manifest.json|archive.zip>\n\nOPTIONS:\n  --version                    Print version and exit\n  --self-check                 Verify native runtime prerequisites and exit\n  --runtime-smoke              Run a bounded runtime binary probe without opening the GUI\n  --runtime-smoke-json         Print runtime smoke probe evidence as JSON\n  --rpc-health                 Check a Neo JSON-RPC endpoint without opening the GUI\n  --rpc-health-json            Print Neo JSON-RPC health probe evidence as JSON\n  --workspace-readiness        Evaluate workspace readiness without opening the GUI\n  --workspace-readiness-json   Print workspace readiness as JSON for CI and scripts\n  --workspace-metrics          Capture system and managed node process metrics\n  --workspace-metrics-json     Print workspace metrics as JSON for CI and scripts\n  --workspace-metrics-prometheus Print workspace metrics in Prometheus text format\n  --workspace-integrity        Run read-only SQLite/schema/foreign-key integrity checks\n  --workspace-integrity-json   Print workspace integrity evidence as JSON\n  --source-purity              Verify the source tree has no Node/Web frontend artifacts\n  --source-purity-json         Print source purity evidence as JSON\n  --source-quality             Verify production markers and Rust file size budgets\n  --source-quality-json        Print source quality evidence as JSON\n  --ci-policy                  Verify CI stays cross-platform, native, and Rust-only\n  --ci-policy-json             Print CI policy evidence as JSON\n  --export-readiness-report    Write text and JSON workspace readiness evidence\n  --export-support-bundle      Write a redacted diagnostics support bundle and ZIP archive\n  --export-support-bundle-json Write support bundle evidence as JSON\n  --export-event-journal       Write filtered redacted text and JSON runtime event audit evidence\n  --export-node-configs        Write all node runtime configs plus export evidence\n  --export-node-configs-json   Write all node runtime configs and print JSON evidence\n  --validate-node-config      Validate a runtime config file against expected node settings\n  --validate-node-config-json Validate a runtime config file and print JSON evidence\n  --export-backup              Export an existing workspace backup without opening the GUI\n  --export-backup-json         Export an existing workspace backup and print JSON evidence\n  --import-backup              Validate and import a workspace backup without opening the GUI\n  --import-backup-json         Import a workspace backup and print JSON evidence\n  --validate-backup            Validate a workspace backup without importing it\n  --validate-backup-json       Print workspace backup validation as JSON\n  --validate-wallet            Validate an encrypted NEP-6 Neo wallet file\n  --validate-wallet-json       Print encrypted NEP-6 wallet validation as JSON\n  --validate-launch-pack       Validate a private-network launch pack without opening the GUI\n  --launch-pack-sidecars       Print supervisor-ready signer sidecar specs from a launch pack\n  --launch-pack-sidecars-json  Print launch pack signer sidecar specs as JSON\n  --package-release            Package the current native executable as a signed-by-checksum ZIP\n  --verify-release-package     Verify release ZIP, manifests, checksum, and binary hash\n  --verify-release-package-json Print release package verification as JSON\n  --help                       Print this help and exit\n\nWithout an option, NeoNexus starts the native desktop application.",
        version = version_text()
    );
    text.push_str("\n\nCONFIG GENERATION:\n  neo-nexus --generate-node-config <neo-cli|neo-go|neo-rs> <mainnet|testnet|private> <leveldb|rocksdb> <rpc-port> <p2p-port> <output-path>\n  neo-nexus --generate-node-config-json <neo-cli|neo-go|neo-rs> <mainnet|testnet|private> <leveldb|rocksdb> <rpc-port> <p2p-port> <output-path>\n");
    text.push_str("\nWALLET PROFILES:\n  neo-nexus --import-wallet-profile <neonexus.db> <wallet.json> <profile-id> <label>\n");
    text.push_str("\nALERT ROUTING:\n  neo-nexus --alert-preview <generic|slack|discord|telegram|pagerduty|opsgenie|datadog> <target-url> <info|warning|critical> <message...>\n  neo-nexus --alert-preview-json <generic|slack|discord|telegram|pagerduty|opsgenie|datadog> <target-url> <info|warning|critical> <message...>\n");
    text.push_str("\nNATIVE BOUNDARY:\n  --source-purity also rejects WebView/Tauri Cargo dependencies, lockfile packages, and project files.\n");
    text.push_str("\nNATIVE UI AUDIT:\n  neo-nexus --native-ui-audit <repo-dir>\n  neo-nexus --native-ui-audit-json <repo-dir>\n");
    text
}

pub(in crate::cli) fn self_check_text() -> Result<String> {
    let check_dir = unique_self_check_dir();
    fs::create_dir_all(&check_dir).with_context(|| {
        format!(
            "failed to create self-check directory {}",
            check_dir.display()
        )
    })?;
    let db_path = check_dir.join("neonexus-self-check.db");
    let repository = Repository::open(db_path.clone()).with_context(|| {
        format!(
            "failed to open self-check SQLite workspace {}",
            db_path.display()
        )
    })?;
    let node_count = repository
        .list_nodes()
        .context("failed to query self-check workspace")?
        .len();
    drop(repository);
    fs::remove_dir_all(&check_dir).with_context(|| {
        format!(
            "failed to remove self-check directory {}",
            check_dir.display()
        )
    })?;

    Ok(format!(
        "{version}\ntarget: {os}/{arch}\nworkspace-db: ok ({node_count} nodes)\nnative-mode: eframe/egui\n",
        version = version_text(),
        os = env::consts::OS,
        arch = env::consts::ARCH,
    ))
}

fn unique_self_check_dir() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    env::temp_dir().join(format!("neonexus-self-check-{}-{timestamp}", process::id()))
}
