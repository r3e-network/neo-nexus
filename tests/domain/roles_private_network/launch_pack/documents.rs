use neo_nexus::private_network::PrivateNetworkDeploymentExport;

pub(super) fn assert_documents(export: &PrivateNetworkDeploymentExport) {
    let start_order = std::fs::read_to_string(&export.start_order_path).unwrap();
    assert!(start_order.contains("neo-rs-validator-1"));
    assert!(start_order.contains("magic: 1230307"));
    assert!(start_order.contains("config_sha256:"));
    let runbook = std::fs::read_to_string(&export.runbook_path).unwrap();
    assert!(runbook.contains("# NeoNexus Private Network Runbook"));
    assert!(runbook.contains("neo-nexus --validate-launch-pack ."));
    assert!(runbook.contains("validation-report.txt"));
    assert!(runbook.contains("validation-report.json"));
    assert!(runbook.contains("Artifact Integrity"));
    assert!(runbook.contains("wallet-provisioning.json"));
    assert!(runbook.contains("Wallet provisioning policy"));
    assert!(runbook.contains("config_sha256="));
    assert!(runbook.contains("Secret Material Boundary"));
    assert!(runbook.contains("never includes private keys"));
    assert!(runbook.contains("sidecar_template=neo-signer --wallet {wallet}"));
    assert!(runbook.contains("sidecar=neo-signer --wallet /secure"));
    assert!(runbook.contains("sidecar_plan=argv-no-shell:neo-signer args=6"));
    assert!(runbook.contains("neo-rs-validator-1"));
    assert!(runbook.contains("powershell -ExecutionPolicy Bypass"));
    let wallet_provisioning: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&export.wallet_provisioning_path).unwrap())
            .unwrap();
    assert_eq!(wallet_provisioning["schema_version"], 1);
    assert_eq!(wallet_provisioning["generated_secret_count"], 0);
    assert_eq!(wallet_provisioning["required_wallet_count"], 4);
    assert_eq!(wallet_provisioning["wallet_reference_count"], 2);
    assert_eq!(wallet_provisioning["missing_wallet_reference_count"], 2);
    assert_eq!(
        wallet_provisioning["entries"][0]["label"],
        "committee-signer-1"
    );
    assert_eq!(
        wallet_provisioning["entries"][0]["wallet_path"],
        "/secure/neonexus/validator-1.wallet.json"
    );
    assert_eq!(
        wallet_provisioning["entries"][0]["path_scope"],
        "posix-absolute"
    );
    assert_eq!(
        wallet_provisioning["entries"][0]["action"],
        "create-or-import-encrypted-wallet-at-referenced-path"
    );
    assert_eq!(
        wallet_provisioning["entries"][0]["signer_command_plan"]["execution_policy"],
        "argv-no-shell"
    );
    assert_eq!(
        wallet_provisioning["entries"][0]["signer_command_plan"]["binary"],
        "neo-signer"
    );
    assert!(wallet_provisioning["entries"][2]["wallet_path"].is_null());
    assert_eq!(
        wallet_provisioning["entries"][2]["recommended_wallet_path"],
        "wallets/committee-signer-3.wallet.json"
    );
    assert_eq!(
        wallet_provisioning["entries"][2]["action"],
        "create-encrypted-wallet-and-add-signer-reference"
    );
    let wallet_instructions = std::fs::read_to_string(&export.wallet_instructions_path).unwrap();
    assert!(wallet_instructions.contains("NeoNexus Wallet Provisioning"));
    assert!(wallet_instructions.contains("Never commit generated wallet files"));
}
