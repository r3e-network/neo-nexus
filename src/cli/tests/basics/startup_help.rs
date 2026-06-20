use super::super::*;

#[test]
fn cli_defaults_to_native_gui() -> Result<()> {
    assert_eq!(action_from_args(["neo-nexus"])?, CliAction::RunGui);
    Ok(())
}

#[test]
fn cli_prints_version_and_help_without_gui() -> Result<()> {
    let version = action_from_args(["neo-nexus", "--version"])?;
    assert!(matches!(version, CliAction::Print(text) if text.contains("NeoNexus")));

    let help = action_from_args(["neo-nexus", "--help"])?;
    let CliAction::Print(help_text) = help else {
        anyhow::bail!("expected help text");
    };
    for expected in [
        "--runtime-smoke-json",
        "--rpc-health-json",
        "--workspace-readiness-json",
        "--workspace-metrics",
        "--workspace-metrics-json",
        "--workspace-metrics-prometheus",
        "--workspace-integrity",
        "--workspace-integrity-json",
        "--source-purity",
        "--source-purity-json",
        "--source-quality",
        "--source-quality-json",
        "--native-ui-audit",
        "--native-ui-audit-json",
        "--ci-policy",
        "--ci-policy-json",
        "--alert-preview",
        "--alert-preview-json",
        "--export-readiness-report",
        "--export-support-bundle",
        "--export-support-bundle-json",
        "--export-event-journal",
        "--export-node-configs",
        "--export-node-configs-json",
        "--generate-node-config",
        "--generate-node-config-json",
        "--validate-node-config",
        "--validate-node-config-json",
        "--export-backup-json",
        "--import-backup-json",
        "--validate-backup-json",
        "--validate-wallet",
        "--validate-wallet-json",
        "--import-wallet-profile",
        "--validate-launch-pack",
        "--launch-pack-sidecars",
        "--launch-pack-sidecars-json",
        "--package-release",
        "--verify-release-package-json",
        "WebView/Tauri Cargo dependencies",
    ] {
        assert!(
            help_text.contains(expected),
            "missing help text: {expected}"
        );
    }
    Ok(())
}
