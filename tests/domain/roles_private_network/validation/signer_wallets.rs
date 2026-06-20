use crate::*;

#[test]
fn launch_pack_validator_rejects_unencrypted_signer_wallet_file() {
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
    let committee_key = committee_public_key("02", 'e');
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
    std::fs::write(&wallet_path, "{}").unwrap();
    write_fake_executable(&export.root_path.join("signer-bin").join("neo-signer"));

    let report = PrivateNetworkLaunchPackVerifier::validate(&export.root_path).unwrap();

    assert!(!report.is_success(), "{}", report.to_cli_text());
    assert!(report.checks.iter().any(|check| {
        check.category == "signer-wallet-format"
            && check.label == "committee-signer-1"
            && check.status == LaunchPackValidationStatus::Fail
            && check.message.contains("NEP-6")
    }));
}

#[test]
fn launch_pack_validator_rejects_signer_wallet_for_different_committee_key() {
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
    let committee_key = committee_public_key("02", 'd');
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

    let report = PrivateNetworkLaunchPackVerifier::validate(&export.root_path).unwrap();

    assert!(!report.is_success(), "{}", report.to_cli_text());
    assert!(report.checks.iter().any(|check| {
        check.category == "signer-wallet-committee-key"
            && check.label == "committee-signer-1"
            && check.status == LaunchPackValidationStatus::Fail
            && check.message.contains(VALID_NEP6_CONTRACT_PUBLIC_KEY)
    }));
}
