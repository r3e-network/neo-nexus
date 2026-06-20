use crate::*;
use neo_nexus::private_network::PrivateNetworkDeploymentExport;

pub(super) fn assert_manifest(export: &PrivateNetworkDeploymentExport) -> String {
    assert_eq!(export.node_count, 7);
    assert_eq!(export.config_count, 7);
    assert_eq!(export.network_magic, 1_230_307);
    assert_eq!(export.validators_count, 4);
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&export.manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["schema_version"], 10);
    assert_eq!(manifest["network_magic"], 1_230_307);
    assert_eq!(manifest["validators_count"], 4);
    assert_eq!(manifest["seed_nodes"][0], "127.0.0.1:30333");
    assert_eq!(manifest["scripts"]["runbook"], "RUNBOOK.md");
    assert_eq!(manifest["scripts"]["preflight_unix"], "preflight-unix.sh");
    assert_eq!(
        manifest["scripts"]["preflight_windows"],
        "preflight-windows.ps1"
    );
    assert_eq!(manifest["scripts"]["health_unix"], "health-unix.sh");
    assert_eq!(manifest["scripts"]["health_windows"], "health-windows.ps1");
    assert_eq!(manifest["scripts"]["start_unix"], "start-unix.sh");
    assert_eq!(manifest["scripts"]["stop_unix"], "stop-unix.sh");
    assert_eq!(manifest["scripts"]["start_windows"], "start-windows.ps1");
    assert_eq!(manifest["scripts"]["stop_windows"], "stop-windows.ps1");
    let artifacts = manifest["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 12);
    let runbook_artifact = artifacts
        .iter()
        .find(|artifact| artifact["label"] == "runbook")
        .unwrap();
    let (runbook_sha256, runbook_bytes) = sha256_file(&export.runbook_path).unwrap();
    assert_eq!(runbook_artifact["path"], "RUNBOOK.md");
    assert_eq!(runbook_artifact["sha256"], runbook_sha256);
    assert_eq!(runbook_artifact["bytes"], runbook_bytes);
    let wallet_provisioning_artifact = artifacts
        .iter()
        .find(|artifact| artifact["label"] == "wallet-provisioning")
        .unwrap();
    let (wallet_provisioning_sha256, wallet_provisioning_bytes) =
        sha256_file(&export.wallet_provisioning_path).unwrap();
    assert_eq!(
        wallet_provisioning_artifact["path"],
        "wallet-provisioning.json"
    );
    assert_eq!(
        wallet_provisioning_artifact["sha256"],
        wallet_provisioning_sha256
    );
    assert_eq!(
        wallet_provisioning_artifact["bytes"],
        wallet_provisioning_bytes
    );
    let wallet_instructions_artifact = artifacts
        .iter()
        .find(|artifact| artifact["label"] == "wallet-instructions")
        .unwrap();
    let (wallet_instructions_sha256, wallet_instructions_bytes) =
        sha256_file(&export.wallet_instructions_path).unwrap();
    assert_eq!(wallet_instructions_artifact["path"], "wallets/README.md");
    assert_eq!(
        wallet_instructions_artifact["sha256"],
        wallet_instructions_sha256
    );
    assert_eq!(
        wallet_instructions_artifact["bytes"],
        wallet_instructions_bytes
    );
    let start_order_artifact = artifacts
        .iter()
        .find(|artifact| artifact["label"] == "start-order")
        .unwrap();
    let (start_order_sha256, start_order_bytes) = sha256_file(&export.start_order_path).unwrap();
    assert_eq!(start_order_artifact["path"], "start-order.txt");
    assert_eq!(start_order_artifact["sha256"], start_order_sha256);
    assert_eq!(start_order_artifact["bytes"], start_order_bytes);
    assert_eq!(manifest["committee"]["signer_count"], 4);
    assert_eq!(manifest["committee"]["wallet_reference_count"], 2);
    assert_eq!(manifest["committee"]["endpoint_reference_count"], 2);
    assert_eq!(manifest["committee"]["sidecar_command_count"], 1);
    assert_eq!(
        manifest["committee"]["secret_material_policy"],
        "references-only-no-private-keys-or-passwords"
    );
    assert_eq!(
        manifest["committee"]["preflight_policy"],
        "check-native-wallet-paths-http-endpoints-and-sidecar-commands"
    );
    assert_eq!(manifest["secret_provisioning"]["schema_version"], 1);
    assert_eq!(
        manifest["secret_provisioning"]["policy"],
        "operator-provided-wallets-no-secret-material-in-launch-pack"
    );
    assert_eq!(
        manifest["secret_provisioning"]["wallet_provisioning_file"],
        "wallet-provisioning.json"
    );
    assert_eq!(
        manifest["secret_provisioning"]["wallet_instructions_file"],
        "wallets/README.md"
    );
    assert_eq!(
        manifest["secret_provisioning"]["recommended_wallet_root"],
        "wallets"
    );
    assert_eq!(manifest["secret_provisioning"]["required_wallet_count"], 4);
    assert_eq!(manifest["secret_provisioning"]["wallet_reference_count"], 2);
    assert_eq!(
        manifest["secret_provisioning"]["missing_wallet_reference_count"],
        2
    );
    assert_eq!(manifest["secret_provisioning"]["generated_secret_count"], 0);
    assert_eq!(
        manifest["committee"]["public_keys"][0],
        committee_public_key("02", '1')
    );
    assert_eq!(
        manifest["committee"]["signers"][0]["wallet_path"],
        "/secure/neonexus/validator-1.wallet.json"
    );
    assert_eq!(
        manifest["committee"]["signers"][0]["signer_endpoint"],
        "http://127.0.0.1:9021"
    );
    assert_eq!(
        manifest["committee"]["signers"][0]["signer_command_template"],
        "neo-signer --wallet {wallet} --listen {endpoint} --label {label}"
    );
    assert_eq!(
        manifest["committee"]["signers"][0]["signer_command"],
        "neo-signer --wallet /secure/neonexus/validator-1.wallet.json --listen http://127.0.0.1:9021 --label committee-signer-1"
    );
    assert_eq!(
        manifest["committee"]["signers"][0]["signer_command_plan"]["execution_policy"],
        "argv-no-shell"
    );
    assert_eq!(
        manifest["committee"]["signers"][0]["signer_command_plan"]["binary"],
        "neo-signer"
    );
    assert_eq!(
        manifest["committee"]["signers"][0]["signer_command_plan"]["arguments"][0],
        "--wallet"
    );
    assert_eq!(
        manifest["committee"]["signers"][0]["signer_command_plan"]["arguments"][1],
        "/secure/neonexus/validator-1.wallet.json"
    );
    assert_eq!(
        manifest["committee"]["signers"][0]["signer_command_plan"]["arguments"][5],
        "committee-signer-1"
    );
    assert_eq!(
        manifest["committee"]["signers"][1]["signer_endpoint"],
        "https://signer.example.test/validator-2"
    );
    assert_eq!(
        manifest["committee"]["signers"][1]["wallet_path"],
        "C:\\neo\\validator-2.wallet.json"
    );
    assert_eq!(manifest["nodes"][0]["role"], "Consensus");
    assert_eq!(manifest["nodes"][0]["binary_path"], "/opt/neo-rs/neo-node");
    assert_eq!(
        manifest["nodes"][0]["arguments"][0].as_str(),
        Some("--config")
    );
    let first_config_path = PathBuf::from(manifest["nodes"][0]["config_path"].as_str().unwrap());
    let (first_config_sha256, _) = sha256_file(&first_config_path).unwrap();
    assert_eq!(
        manifest["nodes"][0]["config_sha256"].as_str(),
        Some(first_config_sha256.as_str())
    );
    assert_eq!(
        manifest["nodes"][0]["config_sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    first_config_sha256
}
