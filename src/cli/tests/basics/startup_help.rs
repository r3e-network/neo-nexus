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
        "maintenance file budgets",
        "hardcoded platform shortcut labels",
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

#[test]
fn cargo_does_not_run_native_gui_binary_as_test_target() -> Result<()> {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = std::fs::read_to_string(manifest_path)?;
    let parsed = toml::from_str::<toml::Value>(&manifest)?;
    let bins = parsed
        .get("bin")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("missing binary targets"))?;
    let neo_nexus_bin = bins
        .iter()
        .find(|bin| bin.get("name").and_then(toml::Value::as_str) == Some("neo-nexus"))
        .ok_or_else(|| anyhow::anyhow!("missing neo-nexus binary target"))?;

    assert_eq!(
        neo_nexus_bin.get("test").and_then(toml::Value::as_bool),
        Some(false)
    );

    let main_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    let main_source = std::fs::read_to_string(main_path)?;
    assert!(
        main_source.contains("#[cfg(not(test))]\nfn main()"),
        "native GUI entrypoint must be disabled in binary test builds"
    );
    assert!(
        main_source.contains("#[cfg(test)]\nfn main() {}"),
        "binary test builds need an empty entrypoint so all-target test listing exits"
    );
    assert!(
        main_source.contains("neo_nexus::manager::action_from_args"),
        "native binary entrypoint should route through the manager mode planner"
    );
    assert!(
        main_source.contains("into_cli_output"),
        "native binary entrypoint should delegate CLI output rendering to manager actions"
    );
    Ok(())
}

#[test]
fn cli_actions_use_core_facade_for_shared_domain_services() -> Result<()> {
    let actions_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli/actions.rs");
    let actions_source = std::fs::read_to_string(actions_path)?;

    assert!(
        actions_source.contains("use crate::core::{"),
        "CLI actions should use the grouped core facade for shared domain services"
    );
    assert!(
        actions_source.contains("quality::{"),
        "CLI actions should use core::quality for native validation services"
    );
    for module in [
        "alerts",
        "backup",
        "ci_policy",
        "config",
        "diagnostics",
        "event_journal_report",
        "events",
        "metrics",
        "native_ui",
        "private_network",
        "readiness_report",
        "release_pack",
        "repository",
        "rpc_health",
        "runtime_smoke",
        "source_purity",
        "source_quality",
        "support_bundle",
        "types",
        "wallet",
        "workspace_integrity",
    ] {
        assert!(
            !actions_source.contains(&format!("use crate::{module}::")),
            "CLI actions should import {module} through src/core/, not directly from crate root"
        );
        assert!(
            !actions_source.contains(&format!("use crate::{{\n    {module}::")),
            "CLI actions should import {module} through src/core/, not directly from crate root"
        );
    }
    Ok(())
}

#[test]
fn cli_quality_output_uses_core_facade_report_types() -> Result<()> {
    let output_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli/output/quality.rs");
    let output_source = std::fs::read_to_string(output_path)?;
    assert!(
        output_source.contains("use crate::core::quality::{"),
        "CLI quality output should use core::quality report types"
    );
    for module in ["ci_policy", "native_ui", "source_purity", "source_quality"] {
        assert!(
            !output_source.contains(&format!("{module}::")),
            "CLI quality output should import {module} reports through core::quality"
        );
    }
    Ok(())
}
