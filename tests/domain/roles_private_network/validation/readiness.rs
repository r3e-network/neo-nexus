use crate::*;

#[path = "readiness/fixture.rs"]
mod fixture;

#[test]
fn private_network_launch_pack_validator_reports_operator_readiness() {
    let fixture = fixture::build_ready_launch_pack();
    let export = fixture.export;
    let signer_binary_path = fixture.signer_binary_path;
    let wallet_path = fixture.wallet_path;

    let ready = PrivateNetworkLaunchPackVerifier::validate(&export.root_path).unwrap();

    assert!(ready.is_success(), "{}", ready.to_cli_text());
    assert_eq!(ready.schema_version, 10);
    assert!(ready
        .checks
        .iter()
        .any(|check| check.category == "signer-wallet"
            && check.label == "committee-signer-1"
            && check.status == LaunchPackValidationStatus::Pass));
    assert!(ready.checks.iter().any(|check| {
        check.category == "signer-wallet-format"
            && check.label == "committee-signer-1"
            && check.status == LaunchPackValidationStatus::Pass
            && check.message.contains("NEP-6 wallet ok")
    }));
    assert!(ready
        .checks
        .iter()
        .any(|check| check.category == "signer-sidecar-binary"
            && check.label == "committee-signer-1"
            && check.status == LaunchPackValidationStatus::Pass
            && check.message.contains("signer-bin")));
    assert!(ready.checks.iter().any(|check| {
        check.category == "signer-sidecar-process-spec"
            && check.label == "committee-signer-1"
            && check.status == LaunchPackValidationStatus::Pass
            && check.message.contains("signer:committee-signer-1")
    }));
    let manifest_sidecars =
        PrivateNetworkLaunchPackVerifier::sidecar_processes(&export.manifest_path).unwrap();
    assert_eq!(manifest_sidecars.len(), 1);
    assert_eq!(
        manifest_sidecars[0].process.binary_path,
        export.root_path.join("signer-bin").join("neo-signer")
    );
    assert_eq!(
        manifest_sidecars[0].process.kind,
        ManagedProcessKind::Sidecar
    );
    assert_eq!(
        manifest_sidecars[0].process.working_dir,
        export.root_path.clone()
    );
    assert_eq!(
        manifest_sidecars[0].log_path,
        export
            .root_path
            .join("signers")
            .join("committee-signer-1")
            .join("committee-signer-1.supervisor.log")
    );
    assert_eq!(
        manifest_sidecars[0].signer_endpoint.as_deref(),
        Some("https://signer.example.test/validator-1")
    );
    assert!(ready.to_cli_text().contains("launch-pack: ok"));
    let ready_report = ready.write_reports().unwrap();
    assert!(ready_report.text_path.is_file());
    assert!(ready_report.json_path.is_file());
    assert!(ready_report.bytes_written > 0);
    assert!(std::fs::read_to_string(&ready_report.text_path)
        .unwrap()
        .contains("launch-pack: ok"));
    let ready_report_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ready_report.json_path).unwrap()).unwrap();
    assert_eq!(ready_report_json["failed_count"], 0);
    assert!(ready_report_json["checks"]
        .as_array()
        .is_some_and(|checks| !checks.is_empty()));

    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&export.manifest_path).unwrap()).unwrap();
    let config_path = PathBuf::from(manifest["nodes"][0]["config_path"].as_str().unwrap());
    let original_config = std::fs::read(&config_path).unwrap();
    let mut tampered_config = original_config.clone();
    tampered_config.extend_from_slice(b"\n# tampered outside NeoNexus\n");
    std::fs::write(&config_path, tampered_config).unwrap();
    let tampered = PrivateNetworkLaunchPackVerifier::validate(&export.root_path).unwrap();

    assert!(!tampered.is_success());
    assert!(tampered
        .checks
        .iter()
        .any(|check| check.category == "config-integrity"
            && check.label == "neo-rs-validator-1 config sha256"
            && check.status == LaunchPackValidationStatus::Fail
            && check.message.contains("expected")));

    std::fs::write(&config_path, &original_config).unwrap();
    let restored = PrivateNetworkLaunchPackVerifier::validate(&export.root_path).unwrap();
    assert!(restored.is_success(), "{}", restored.to_cli_text());

    let original_start_unix = std::fs::read(&export.start_unix_path).unwrap();
    let mut tampered_start_unix = original_start_unix.clone();
    tampered_start_unix.extend_from_slice(b"\n# tampered start script\n");
    std::fs::write(&export.start_unix_path, tampered_start_unix).unwrap();
    let tampered_artifact = PrivateNetworkLaunchPackVerifier::validate(&export.root_path).unwrap();

    assert!(!tampered_artifact.is_success());
    assert!(tampered_artifact
        .checks
        .iter()
        .any(|check| check.category == "artifact-integrity"
            && check.label == "start-unix sha256"
            && check.status == LaunchPackValidationStatus::Fail
            && check.message.contains("actual")));

    std::fs::write(&export.start_unix_path, &original_start_unix).unwrap();
    let restored_artifacts = PrivateNetworkLaunchPackVerifier::validate(&export.root_path).unwrap();
    assert!(
        restored_artifacts.is_success(),
        "{}",
        restored_artifacts.to_cli_text()
    );

    std::fs::remove_file(&signer_binary_path).unwrap();
    let missing_signer_binary =
        PrivateNetworkLaunchPackVerifier::validate(&export.manifest_path).unwrap();
    assert!(!missing_signer_binary.is_success());
    assert!(missing_signer_binary.checks.iter().any(|check| {
        check.category == "signer-sidecar-binary"
            && check.label == "committee-signer-1"
            && check.status == LaunchPackValidationStatus::Fail
            && check.message.contains("sidecar binary missing")
    }));
    write_fake_executable(&signer_binary_path);

    std::fs::remove_file(&wallet_path).unwrap();
    let missing_wallet = PrivateNetworkLaunchPackVerifier::validate(&export.manifest_path).unwrap();

    assert!(!missing_wallet.is_success());
    assert!(missing_wallet
        .checks
        .iter()
        .any(|check| check.category == "signer-wallet"
            && check.label == "committee-signer-1"
            && check.status == LaunchPackValidationStatus::Fail));
    assert!(missing_wallet.to_cli_text().contains("launch-pack: failed"));
    let missing_report = missing_wallet.write_reports().unwrap();
    assert!(std::fs::read_to_string(&missing_report.text_path)
        .unwrap()
        .contains("launch-pack: failed"));
    let missing_report_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&missing_report.json_path).unwrap()).unwrap();
    assert!(missing_report_json["failed_count"].as_u64().unwrap_or(0) > 0);
}
