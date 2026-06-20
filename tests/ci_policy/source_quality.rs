use super::*;

#[test]
fn ci_policy_requires_source_quality_for_test_sources() -> anyhow::Result<()> {
    let workflow = r#"
name: Rust CI

jobs:
  verify:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - run: cargo fmt --all --check
      - run: cargo check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --lib
      - run: cargo test --test ci_policy
      - run: cargo test --test domain
      - run: cargo test --test repository
      - run: cargo run -- --self-check
      - run: cargo run -- --ci-policy .github/workflows/ci.yml
      - run: cargo run -- --ci-policy-json .github/workflows/ci.yml
      - run: cargo run -- --source-purity .
      - run: cargo run -- --source-purity-json .
      - run: cargo run -- --source-quality src
      - run: cargo run -- --source-quality-json src
      - run: cargo run -- --native-ui-audit .
      - run: cargo run -- --native-ui-audit-json .
      - run: cargo run -- --alert-preview datadog https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123 critical "RPC health unreachable"
      - run: cargo run -- --alert-preview-json datadog https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123 critical "RPC health unreachable"
      - run: |
          grep 'alert-preview: ready' /tmp/alert.txt
          grep 'DD-API-KEY=<redacted>' /tmp/alert.txt
          grep '"provider": "datadog"' /tmp/alert.json
      - run: cargo run -- --runtime-smoke neo-rs /definitely/missing/neo-node
      - run: cargo run -- --runtime-smoke-json neo-rs /tmp/neo-node
      - run: |
          grep 'runtime-smoke: passed' /tmp/runtime-smoke.txt
          grep 'runtime-binary-sha256:' /tmp/runtime-smoke.txt
          grep '"status": "passed"' /tmp/runtime-smoke.json
          grep '"binary_evidence"' /tmp/runtime-smoke.json
          grep '"status": "verified"' /tmp/runtime-smoke.json
          grep '"sha256":' /tmp/runtime-smoke.json
      - run: cargo run -- --rpc-health 127.0.0.1:1
      - run: cargo run -- --rpc-health-json 127.0.0.1:1
      - run: cargo run -- --workspace-readiness /tmp/neonexus.db
      - run: cargo run -- --workspace-metrics /tmp/neonexus.db
      - run: cargo run -- --workspace-metrics-json /tmp/neonexus.db
      - run: cargo run -- --workspace-metrics-prometheus /tmp/neonexus.db
      - run: cargo run -- --workspace-integrity /tmp/neonexus.db
      - run: cargo run -- --export-readiness-report /tmp/neonexus.db /tmp/reports
      - run: cargo run -- --export-support-bundle /tmp/neonexus.db /tmp/support
      - run: cargo run -- --export-support-bundle-json /tmp/neonexus.db /tmp/support-json
      - run: cargo run -- --export-event-journal /tmp/neonexus.db /tmp/events
      - run: cargo run -- --export-node-configs /tmp/neonexus.db /tmp/configs
      - run: cargo run -- --export-node-configs-json /tmp/neonexus.db /tmp/configs-json
      - run: cargo run -- --generate-node-config neo-rs testnet rocksdb 20332 20333 /tmp/generated-neo-rs.toml
      - run: cargo run -- --generate-node-config-json neo-rs testnet rocksdb 20342 20343 /tmp/generated-neo-rs-json.toml
      - run: cargo run -- --validate-node-config neo-rs testnet rocksdb 20332 20333 /tmp/generated-neo-rs.toml
      - run: cargo run -- --validate-node-config-json neo-rs testnet rocksdb 20342 20343 /tmp/generated-neo-rs-json.toml
      - run: cargo run -- --export-backup /tmp/neonexus.db /tmp/backups
      - run: cargo run -- --export-backup-json /tmp/neonexus.db /tmp/backups-json
      - run: cargo run -- --validate-backup /tmp/neonexus-backup.json
      - run: cargo run -- --validate-backup-json /tmp/neonexus-backup.json
      - run: cargo run -- --import-backup /tmp/import.db /tmp/neonexus-backup.json
      - run: cargo run -- --import-backup-json /tmp/import-json.db /tmp/neonexus-backup.json
      - run: cargo run -- --validate-wallet /tmp/validator.wallet.json
      - run: cargo run -- --validate-wallet-json /tmp/validator.wallet.json
      - run: cargo run -- --import-wallet-profile /tmp/neonexus.db /tmp/validator.wallet.json validator-wallet-1 "Validator wallet 1"
      - run: cargo build --release
      - run: ./target/release/neo-nexus --self-check
      - run: .\target\release\neo-nexus.exe --self-check
      - run: ./target/release/neo-nexus --package-release dist
      - run: ./target/release/neo-nexus --verify-release-package dist
      - run: ./target/release/neo-nexus --verify-release-package-json dist
"#;

    let report =
        CiPolicyChecker::check_text_at(".github/workflows/ci.yml", workflow, 1_800_000_000)?;

    assert!(!report.is_success());
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "source-quality-tests-text"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "source-quality-tests-json"));

    Ok(())
}

#[test]
fn ci_policy_requires_workspace_source_quality_for_docs_and_ci_budget() -> anyhow::Result<()> {
    let workflow = r#"
name: Rust CI

jobs:
  verify:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - run: cargo fmt --all --check
      - run: cargo check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --lib
      - run: cargo test --test ci_policy
      - run: cargo test --test domain
      - run: cargo test --test repository
      - run: cargo run -- --self-check
      - run: cargo run -- --ci-policy .github/workflows/ci.yml
      - run: cargo run -- --ci-policy-json .github/workflows/ci.yml
      - run: cargo run -- --source-purity .
      - run: cargo run -- --source-purity-json .
      - run: cargo run -- --source-quality src
      - run: cargo run -- --source-quality-json src
      - run: cargo run -- --source-quality tests
      - run: cargo run -- --source-quality-json tests
      - run: cargo run -- --native-ui-audit .
      - run: cargo run -- --native-ui-audit-json .
      - run: cargo run -- --alert-preview datadog https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123 critical "RPC health unreachable"
      - run: cargo run -- --alert-preview-json datadog https://event-management-intake.datadoghq.com/api/v2/events?api_key=dd123 critical "RPC health unreachable"
      - run: |
          grep 'alert-preview: ready' /tmp/alert.txt
          grep 'DD-API-KEY=<redacted>' /tmp/alert.txt
          grep '"provider": "datadog"' /tmp/alert.json
      - run: cargo run -- --runtime-smoke neo-rs /definitely/missing/neo-node
      - run: cargo run -- --runtime-smoke-json neo-rs /tmp/neo-node
      - run: |
          grep 'runtime-smoke: passed' /tmp/runtime-smoke.txt
          grep 'runtime-binary-sha256:' /tmp/runtime-smoke.txt
          grep '"status": "passed"' /tmp/runtime-smoke.json
          grep '"binary_evidence"' /tmp/runtime-smoke.json
          grep '"status": "verified"' /tmp/runtime-smoke.json
          grep '"sha256":' /tmp/runtime-smoke.json
      - run: cargo run -- --rpc-health 127.0.0.1:1
      - run: cargo run -- --rpc-health-json 127.0.0.1:1
      - run: cargo run -- --workspace-readiness /tmp/neonexus.db
      - run: cargo run -- --workspace-metrics /tmp/neonexus.db
      - run: cargo run -- --workspace-metrics-json /tmp/neonexus.db
      - run: cargo run -- --workspace-metrics-prometheus /tmp/neonexus.db
      - run: cargo run -- --workspace-integrity /tmp/neonexus.db
      - run: cargo run -- --export-readiness-report /tmp/neonexus.db /tmp/reports
      - run: cargo run -- --export-support-bundle /tmp/neonexus.db /tmp/support
      - run: cargo run -- --export-support-bundle-json /tmp/neonexus.db /tmp/support-json
      - run: cargo run -- --export-event-journal /tmp/neonexus.db /tmp/events
      - run: cargo run -- --export-node-configs /tmp/neonexus.db /tmp/configs
      - run: cargo run -- --export-node-configs-json /tmp/neonexus.db /tmp/configs-json
      - run: cargo run -- --generate-node-config neo-rs testnet rocksdb 20332 20333 /tmp/generated-neo-rs.toml
      - run: cargo run -- --generate-node-config-json neo-rs testnet rocksdb 20342 20343 /tmp/generated-neo-rs-json.toml
      - run: cargo run -- --validate-node-config neo-rs testnet rocksdb 20332 20333 /tmp/generated-neo-rs.toml
      - run: cargo run -- --validate-node-config-json neo-rs testnet rocksdb 20342 20343 /tmp/generated-neo-rs-json.toml
      - run: cargo run -- --export-backup /tmp/neonexus.db /tmp/backups
      - run: cargo run -- --export-backup-json /tmp/neonexus.db /tmp/backups-json
      - run: cargo run -- --validate-backup /tmp/neonexus-backup.json
      - run: cargo run -- --validate-backup-json /tmp/neonexus-backup.json
      - run: cargo run -- --import-backup /tmp/import.db /tmp/neonexus-backup.json
      - run: cargo run -- --import-backup-json /tmp/import-json.db /tmp/neonexus-backup.json
      - run: cargo run -- --validate-wallet /tmp/validator.wallet.json
      - run: cargo run -- --validate-wallet-json /tmp/validator.wallet.json
      - run: cargo run -- --import-wallet-profile /tmp/neonexus.db /tmp/validator.wallet.json validator-wallet-1 "Validator wallet 1"
      - run: cargo build --release
      - run: ./target/release/neo-nexus --self-check
      - run: .\target\release\neo-nexus.exe --self-check
      - run: ./target/release/neo-nexus --package-release dist
      - run: ./target/release/neo-nexus --verify-release-package dist
      - run: ./target/release/neo-nexus --verify-release-package-json dist
"#;

    let report =
        CiPolicyChecker::check_text_at(".github/workflows/ci.yml", workflow, 1_800_000_000)?;

    assert!(!report.is_success());
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "source-quality-root-text"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "source-quality-root-json"));

    Ok(())
}
