use super::*;

#[test]
fn ci_policy_rejects_missing_platforms_native_gates_and_frontend_tooling() -> anyhow::Result<()> {
    let workflow = r#"
name: Partial CI

jobs:
  verify:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/setup-node@v4
      - run: npm run build
      - run: cargo check
"#;

    let report =
        CiPolicyChecker::check_text_at(".github/workflows/ci.yml", workflow, 1_800_000_000)?;

    assert!(!report.is_success());
    assert_eq!(report.status, "failed");
    assert!(report.missing_os.iter().any(|os| os == "windows-latest"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "source-quality-src-text"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "source-quality-tests-text"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "workspace-metrics-text"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "workspace-metrics-prometheus"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "native-ui-audit-text"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "alert-preview-json"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "neo-rs-runtime-smoke-json-passed"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "neo-rs-runtime-smoke-json-binary-evidence"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "neo-rs-runtime-smoke-json-sha256"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "release-self-check-windows"));
    assert!(report.findings.iter().any(|finding| {
        finding.category == "forbidden-ci-tooling" && finding.marker == "actions/setup-node"
    }));
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.category == "missing-command"));

    Ok(())
}

#[test]
fn ci_policy_rejects_ambiguous_integration_test_target() -> anyhow::Result<()> {
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
      - run: cargo test --tests
"#;

    let report =
        CiPolicyChecker::check_text_at(".github/workflows/ci.yml", workflow, 1_800_000_000)?;

    assert!(!report.is_success());
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "cargo-test-ci-policy"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "cargo-test-domain"));
    assert!(report
        .missing_commands
        .iter()
        .any(|command| command == "cargo-test-repository"));

    Ok(())
}
