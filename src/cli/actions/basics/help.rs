use super::version_text;

pub(in crate::cli) fn help_text() -> String {
    [
        version_text(),
        help_section(USAGE_LINES),
        help_section(APPLICATION_MODE_LINES),
        help_section(OPTION_LINES),
        help_section(CONFIG_GENERATION_LINES),
        help_section(WALLET_PROFILE_LINES),
        help_section(ALERT_ROUTING_LINES),
        help_section(NATIVE_BOUNDARY_LINES),
        help_section(SOURCE_QUALITY_LINES),
        help_section(NATIVE_UI_AUDIT_LINES),
    ]
    .join("\n\n")
}

fn help_section(lines: &[&str]) -> String {
    lines.join("\n")
}

const USAGE_LINES: &[&str] = &[
    "USAGE:",
    "  neo-nexus [--gui|--version|--self-check|--help]",
    "  neo-nexus --completions <bash|zsh|fish>",
    "  neo-nexus --runtime-smoke <neo-cli|neo-go|neo-rs> <binary> [runtime-args...]",
    "  neo-nexus --runtime-smoke-json <neo-cli|neo-go|neo-rs> <binary> [runtime-args...]",
    "  neo-nexus --rpc-health <port|url>",
    "  neo-nexus --rpc-health-json <port|url>",
    "  neo-nexus --workspace-readiness <neonexus.db>",
    "  neo-nexus --workspace-readiness-json <neonexus.db>",
    "  neo-nexus --workspace-metrics <neonexus.db>",
    "  neo-nexus --workspace-metrics-json <neonexus.db>",
    "  neo-nexus --workspace-metrics-prometheus <neonexus.db>",
    "  neo-nexus --workspace-integrity <neonexus.db>",
    "  neo-nexus --workspace-integrity-json <neonexus.db>",
    "  neo-nexus --source-purity <repo-dir>",
    "  neo-nexus --source-purity-json <repo-dir>",
    "  neo-nexus --source-quality <source-dir>",
    "  neo-nexus --source-quality-json <source-dir>",
    "  neo-nexus --ci-policy <workflow.yml>",
    "  neo-nexus --ci-policy-json <workflow.yml>",
    "  neo-nexus --export-readiness-report <neonexus.db> <output-dir>",
    "  neo-nexus --export-support-bundle <neonexus.db> <output-dir>",
    "  neo-nexus --export-support-bundle-json <neonexus.db> <output-dir>",
    "  neo-nexus --export-event-journal <neonexus.db> <output-dir> [limit] [info|warning|critical|all] [query...]",
    "  neo-nexus --export-node-configs <neonexus.db> <output-dir>",
    "  neo-nexus --export-node-configs-json <neonexus.db> <output-dir>",
    "  neo-nexus --validate-node-config <neo-cli|neo-go|neo-rs> <mainnet|testnet|private> <leveldb|rocksdb> <rpc-port> <p2p-port> <config-path>",
    "  neo-nexus --validate-node-config-json <neo-cli|neo-go|neo-rs> <mainnet|testnet|private> <leveldb|rocksdb> <rpc-port> <p2p-port> <config-path>",
    "  neo-nexus --export-backup <neonexus.db> <output-dir>",
    "  neo-nexus --export-backup-json <neonexus.db> <output-dir>",
    "  neo-nexus --import-backup <neonexus.db> <backup.json>",
    "  neo-nexus --import-backup-json <neonexus.db> <backup.json>",
    "  neo-nexus --validate-backup <backup.json>",
    "  neo-nexus --validate-backup-json <backup.json>",
    "  neo-nexus --validate-wallet <wallet.json>",
    "  neo-nexus --validate-wallet-json <wallet.json>",
    "  neo-nexus --validate-launch-pack <manifest.json|launch-pack-dir>",
    "  neo-nexus --launch-pack-sidecars <manifest.json|launch-pack-dir>",
    "  neo-nexus --launch-pack-sidecars-json <manifest.json|launch-pack-dir>",
    "  neo-nexus --package-release <output-dir>",
    "  neo-nexus --verify-release-package <dist-dir|manifest.json|archive.zip>",
    "  neo-nexus --verify-release-package-json <dist-dir|manifest.json|archive.zip>",
];

const APPLICATION_MODE_LINES: &[&str] = &[
    "APPLICATION MODE:",
    "  neo-nexus           Start the native desktop application",
    "  neo-nexus --gui     Start the native desktop application explicitly",
    "  all other options   Run a headless manager command and exit",
    "Without an option, NeoNexus starts the native desktop application.",
];

const OPTION_LINES: &[&str] = &[
    "OPTIONS:",
    "  --gui                        Start the native desktop application",
    "  --version                    Print version and exit",
    "  --self-check                 Verify native runtime prerequisites and exit",
    "  --completions                Print a shell completion script (bash, zsh, or fish)",
    "  --runtime-smoke              Run a bounded runtime binary probe without opening the GUI",
    "  --runtime-smoke-json         Print runtime smoke probe evidence as JSON",
    "  --rpc-health                 Check a Neo JSON-RPC endpoint without opening the GUI",
    "  --rpc-health-json            Print Neo JSON-RPC health probe evidence as JSON",
    "  --workspace-readiness        Evaluate workspace readiness without opening the GUI",
    "  --workspace-readiness-json   Print workspace readiness as JSON for CI and scripts",
    "  --workspace-metrics          Capture system and managed node process metrics",
    "  --workspace-metrics-json     Print workspace metrics as JSON for CI and scripts",
    "  --workspace-metrics-prometheus Print workspace metrics in Prometheus text format",
    "  --workspace-integrity        Run read-only SQLite/schema/foreign-key integrity checks",
    "  --workspace-integrity-json   Print workspace integrity evidence as JSON",
    "  --source-purity              Verify the source tree has no Node/Web frontend artifacts",
    "  --source-purity-json         Print source purity evidence as JSON",
    "  --source-quality             Verify production markers and Rust file size budgets",
    "  --source-quality-json        Print source quality evidence as JSON",
    "  --ci-policy                  Verify CI stays cross-platform, native, and Rust-only",
    "  --ci-policy-json             Print CI policy evidence as JSON",
    "  --export-readiness-report    Write text and JSON workspace readiness evidence",
    "  --export-support-bundle      Write a redacted diagnostics support bundle and ZIP archive",
    "  --export-support-bundle-json Write support bundle evidence as JSON",
    "  --export-event-journal       Write filtered redacted text and JSON runtime event audit evidence",
    "  --export-node-configs        Write all node runtime configs plus export evidence",
    "  --export-node-configs-json   Write all node runtime configs and print JSON evidence",
    "  --validate-node-config       Validate a runtime config file against expected node settings",
    "  --validate-node-config-json  Validate a runtime config file and print JSON evidence",
    "  --export-backup              Export an existing workspace backup without opening the GUI",
    "  --export-backup-json         Export an existing workspace backup and print JSON evidence",
    "  --import-backup              Validate and import a workspace backup without opening the GUI",
    "  --import-backup-json         Import a workspace backup and print JSON evidence",
    "  --validate-backup            Validate a workspace backup without importing it",
    "  --validate-backup-json       Print workspace backup validation as JSON",
    "  --validate-wallet            Validate an encrypted NEP-6 Neo wallet file",
    "  --validate-wallet-json       Print encrypted NEP-6 wallet validation as JSON",
    "  --validate-launch-pack       Validate a private-network launch pack without opening the GUI",
    "  --launch-pack-sidecars       Print supervisor-ready signer sidecar specs from a launch pack",
    "  --launch-pack-sidecars-json  Print launch pack signer sidecar specs as JSON",
    "  --package-release            Package the current native executable as a signed-by-checksum ZIP",
    "  --verify-release-package     Verify release ZIP, manifests, checksum, and binary hash",
    "  --verify-release-package-json Print release package verification as JSON",
    "  --help                       Print this help and exit",
];

const CONFIG_GENERATION_LINES: &[&str] = &[
    "CONFIG GENERATION:",
    "  neo-nexus --generate-node-config <neo-cli|neo-go|neo-rs> <mainnet|testnet|private> <leveldb|rocksdb> <rpc-port> <p2p-port> <output-path>",
    "  neo-nexus --generate-node-config-json <neo-cli|neo-go|neo-rs> <mainnet|testnet|private> <leveldb|rocksdb> <rpc-port> <p2p-port> <output-path>",
];

const WALLET_PROFILE_LINES: &[&str] = &[
    "WALLET PROFILES:",
    "  neo-nexus --import-wallet-profile <neonexus.db> <wallet.json> <profile-id> <label>",
];

const ALERT_ROUTING_LINES: &[&str] = &[
    "ALERT ROUTING:",
    "  neo-nexus --alert-preview <generic|slack|discord|telegram|pagerduty|opsgenie|datadog> <target-url> <info|warning|critical> <message...>",
    "  neo-nexus --alert-preview-json <generic|slack|discord|telegram|pagerduty|opsgenie|datadog> <target-url> <info|warning|critical> <message...>",
];

const NATIVE_BOUNDARY_LINES: &[&str] = &[
    "NATIVE BOUNDARY:",
    "  --source-purity also rejects WebView/Tauri Cargo dependencies, lockfile packages, and project files.",
];

const SOURCE_QUALITY_LINES: &[&str] = &[
    "SOURCE QUALITY:",
    "  --source-quality checks production markers, hardcoded platform shortcut labels, Rust module budgets, and repository maintenance file budgets.",
];

const NATIVE_UI_AUDIT_LINES: &[&str] = &[
    "NATIVE UI AUDIT:",
    "  neo-nexus --native-ui-audit <repo-dir>",
    "  neo-nexus --native-ui-audit-json <repo-dir>",
];
