use std::path::PathBuf;

use neo_nexus::private_network::PrivateNetworkDeploymentExport;

use super::*;

pub(super) struct LaunchPackFixture {
    pub(super) _temp_dir: tempfile::TempDir,
    pub(super) export: PrivateNetworkDeploymentExport,
    pub(super) signer_binary_path: PathBuf,
    pub(super) wallet_path: PathBuf,
}

pub(super) fn build_ready_launch_pack() -> LaunchPackFixture {
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
            binary_path: fake_binary.clone(),
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
    let signer_binary_path = export.root_path.join("signer-bin").join("neo-signer");
    write_fake_executable(&signer_binary_path);
    LaunchPackFixture {
        _temp_dir: temp_dir,
        export,
        signer_binary_path,
        wallet_path,
    }
}
