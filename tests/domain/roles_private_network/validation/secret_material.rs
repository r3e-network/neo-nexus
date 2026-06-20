use crate::*;

#[test]
fn launch_pack_validator_rejects_wallet_provisioning_secret_material() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let fake_binary = temp_dir.path().join("bin").join("neo-node");
    std::fs::create_dir_all(fake_binary.parent().unwrap()).unwrap();
    File::create(&fake_binary).unwrap();

    let template_id = repo
        .create_node(NewNode {
            name: "neo-rs template".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: fake_binary,
            args: Vec::new(),
            runtime_version: "v0.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap()
        .id;
    let template = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == template_id)
        .unwrap();
    let plan =
        PrivateNetworkPlanner::plan(PrivateNetworkTemplate::SingleValidator, NodeType::NeoRs);
    let created = repo
        .create_nodes_with_plugins(
            plan.to_new_nodes(&template)
                .unwrap()
                .into_iter()
                .map(|input| (input, Vec::new()))
                .collect(),
        )
        .unwrap();
    let committee_key = VALID_NEP6_CONTRACT_PUBLIC_KEY.to_string();
    let request = PrivateNetworkDeploymentRequest {
        plan,
        nodes: created,
        plugin_states: BTreeMap::new(),
        committee: CommitteeRoster::from_public_keys_and_references(
            &committee_key,
            &format!(
                "{committee_key}|wallets/validator-1.wallet.json|https://signer.example.test/validator-1|signer-bin/neo-signer --wallet {{wallet}} --listen {{endpoint}}"
            ),
        )
        .unwrap(),
        output_dir: temp_dir.path().join("private-networks"),
        node_root_dir: temp_dir.path().join("nodes"),
    };
    let export = PrivateNetworkDeploymentExporter::write(request).unwrap();
    let wallet_path = export
        .root_path
        .join("wallets")
        .join("validator-1.wallet.json");
    std::fs::create_dir_all(wallet_path.parent().unwrap()).unwrap();
    std::fs::write(&wallet_path, valid_nep6_wallet_json()).unwrap();
    write_fake_executable(&export.root_path.join("signer-bin").join("neo-signer"));
    let ready = PrivateNetworkLaunchPackVerifier::validate(&export.root_path).unwrap();
    assert!(ready.is_success(), "{}", ready.to_cli_text());

    let mut wallet_provisioning: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&export.wallet_provisioning_path).unwrap())
            .unwrap();
    wallet_provisioning
        .as_object_mut()
        .unwrap()
        .insert("password".to_string(), serde_json::json!("raw-password"));
    wallet_provisioning["entries"].as_array_mut().unwrap()[0]
        .as_object_mut()
        .unwrap()
        .insert(
            "private_key".to_string(),
            serde_json::json!("Kzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"),
        );
    let tampered_wallet_provisioning = serde_json::to_string_pretty(&wallet_provisioning).unwrap();
    std::fs::write(
        &export.wallet_provisioning_path,
        tampered_wallet_provisioning.as_bytes(),
    )
    .unwrap();

    let (wallet_provisioning_sha256, wallet_provisioning_bytes) =
        sha256_file(&export.wallet_provisioning_path).unwrap();
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&export.manifest_path).unwrap()).unwrap();
    let artifacts = manifest["artifacts"].as_array_mut().unwrap();
    let artifact = artifacts
        .iter_mut()
        .find(|artifact| artifact["label"] == "wallet-provisioning")
        .unwrap();
    artifact["sha256"] = serde_json::Value::String(wallet_provisioning_sha256);
    artifact["bytes"] =
        serde_json::Value::Number(serde_json::Number::from(wallet_provisioning_bytes));
    std::fs::write(
        &export.manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let tampered = PrivateNetworkLaunchPackVerifier::validate(&export.root_path).unwrap();

    assert!(!tampered.is_success(), "{}", tampered.to_cli_text());
    assert!(tampered.checks.iter().any(|check| {
        check.category == "secret-provisioning"
            && check.label == "wallet-provisioning-secret-boundary"
            && check.status == LaunchPackValidationStatus::Fail
            && check.message.contains("private_key")
            && check.message.contains("password")
    }));
}
