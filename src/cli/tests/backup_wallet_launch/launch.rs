use super::super::*;

#[test]
fn launch_pack_cli_reports_validation_failure_status() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let manifest_path = temp_dir.path().join("manifest.json");
    std::fs::write(
        &manifest_path,
        r#"{
          "schema_version": 5,
          "generated_at_unix": 1800000000,
          "template": "Single validator",
          "runtime": "neo-rs",
          "network": "private",
          "network_magic": 1230301,
          "validators_count": 1,
          "seed_nodes": ["127.0.0.1:30333"],
          "committee": {
            "signer_count": 0,
            "wallet_reference_count": 0,
            "endpoint_reference_count": 0,
            "public_keys": [],
            "secret_material_policy": "references-only-no-private-keys-or-passwords",
            "preflight_policy": "check-native-wallet-paths-and-http-endpoints",
            "signers": []
          },
          "scripts": {
            "runbook": "RUNBOOK.md",
            "preflight_unix": "preflight-unix.sh",
            "preflight_windows": "preflight-windows.ps1",
            "health_unix": "health-unix.sh",
            "health_windows": "health-windows.ps1",
            "start_unix": "start-unix.sh",
            "stop_unix": "stop-unix.sh",
            "start_windows": "start-windows.ps1",
            "stop_windows": "stop-windows.ps1"
          },
          "nodes": []
        }"#,
    )?;

    let manifest_arg = manifest_path.display().to_string();
    let action = action_from_args(["neo-nexus", "--validate-launch-pack", &manifest_arg])?;

    assert!(
        matches!(action, CliAction::PrintWithExitCode { text, exit_code: 1 } if text.contains("launch-pack: failed") && text.contains("report-text:") && text.contains("report-json:"))
    );
    assert!(temp_dir.path().join("validation-report.txt").is_file());
    assert!(temp_dir.path().join("validation-report.json").is_file());
    Ok(())
}

#[test]
fn launch_pack_sidecars_cli_reports_supervisor_specs() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let manifest_path = write_launch_pack_sidecar_manifest(temp_dir.path())?;
    let manifest_arg = manifest_path.display().to_string();

    let action = action_from_args(["neo-nexus", "--launch-pack-sidecars", &manifest_arg])?;

    assert!(
        matches!(action, CliAction::PrintWithExitCode { text, exit_code: 0 } if text.contains("launch-pack-sidecars: planned")
            && text.contains("sidecars: 1")
            && text.contains("signer:committee-signer-1")
            && text.contains("binary:")
            && text.contains("signer-bin/neo-signer")
            && text.contains("working-dir:")
            && text.contains("committee-signer-1.supervisor.log")
            && text.contains("endpoint: http://127.0.0.1:9021"))
    );
    Ok(())
}

#[test]
fn launch_pack_sidecars_json_cli_reports_machine_readable_specs() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let manifest_path = write_launch_pack_sidecar_manifest(temp_dir.path())?;
    let manifest_arg = manifest_path.display().to_string();

    let action = action_from_args(["neo-nexus", "--launch-pack-sidecars-json", &manifest_arg])?;

    let CliAction::PrintWithExitCode { text, exit_code: 0 } = action else {
        anyhow::bail!("expected JSON launch pack sidecar action");
    };
    let value: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["status"], "planned");
    assert_eq!(value["report"]["manifest_path"], manifest_arg);
    assert_eq!(value["report"]["sidecar_count"], 1);
    assert_eq!(
        value["report"]["sidecars"][0]["signer_label"],
        "committee-signer-1"
    );
    assert_eq!(value["report"]["sidecars"][0]["process"]["kind"], "sidecar");
    assert_eq!(
        value["report"]["sidecars"][0]["process"]["id"],
        "signer:committee-signer-1"
    );
    assert_eq!(
        value["report"]["sidecars"][0]["process"]["args"][1],
        "wallets/validator-1.wallet.json"
    );
    assert!(value["report"]["sidecars"][0]["log_path"]
        .as_str()
        .unwrap_or_default()
        .ends_with("signers/committee-signer-1/committee-signer-1.supervisor.log"));
    Ok(())
}
