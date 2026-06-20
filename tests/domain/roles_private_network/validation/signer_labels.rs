use crate::*;

#[test]
fn launch_pack_validator_rejects_duplicate_committee_signer_labels() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");
    let first_key = committee_public_key("02", '1');
    let second_key = committee_public_key("03", '2');
    let manifest = serde_json::json!({
        "schema_version": 10,
        "generated_at_unix": 1,
        "template": "single-validator",
        "runtime": "neo-rs",
        "network": Network::Private.to_string(),
        "network_magic": 1_230_307,
        "validators_count": 2,
        "seed_nodes": [],
        "committee": {
            "signer_count": 2,
            "wallet_reference_count": 0,
            "endpoint_reference_count": 0,
            "sidecar_command_count": 0,
            "public_keys": [first_key, second_key],
            "secret_material_policy": "references-only-no-private-keys-or-passwords",
            "preflight_policy": "check-native-wallet-paths-http-endpoints-and-sidecar-commands",
            "signers": [
                {
                    "label": "committee-signer-1",
                    "public_key": first_key
                },
                {
                    "label": "Committee-Signer-1",
                    "public_key": second_key
                }
            ]
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
        "artifacts": [],
        "nodes": []
    });
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let report = PrivateNetworkLaunchPackVerifier::validate(&manifest_path).unwrap();

    assert!(!report.is_success());
    assert!(report.checks.iter().any(|check| {
        check.category == "committee"
            && check.label == "signer-labels"
            && check.status == LaunchPackValidationStatus::Fail
            && check.message.contains("duplicates: committee-signer-1 x2")
    }));
}
