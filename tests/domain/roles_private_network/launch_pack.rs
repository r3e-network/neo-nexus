use crate::*;

#[path = "launch_pack/configs.rs"]
mod configs;
#[path = "launch_pack/documents.rs"]
mod documents;
#[path = "launch_pack/manifest_assertions.rs"]
mod manifest_assertions;
#[path = "launch_pack/scripts.rs"]
mod scripts;

#[test]
fn private_network_launch_pack_writes_managed_configs_and_manifest() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let template_id = repo
        .create_node(NewNode {
            name: "neo-rs template".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: PathBuf::from("/opt/neo-rs/neo-node"),
            args: vec!["--config".to_string(), "testnet.toml".to_string()],
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
    let plan = PrivateNetworkPlanner::plan(PrivateNetworkTemplate::SevenNodeLab, NodeType::NeoRs);
    let inputs = plan.to_new_nodes(&template).unwrap();
    let created = repo
        .create_nodes_with_plugins(
            inputs
                .into_iter()
                .map(|input| (input, Vec::new()))
                .collect(),
        )
        .unwrap();
    let committee_key_values = [
        committee_public_key("02", '1'),
        committee_public_key("03", '2'),
        committee_public_key("02", '3'),
        committee_public_key("03", '4'),
    ];
    let committee_keys = committee_key_values.join(",");
    let signer_refs = format!(
        "{}|/secure/neonexus/validator-1.wallet.json|http://127.0.0.1:9021|neo-signer --wallet {{wallet}} --listen {{endpoint}} --label {{label}}\n{}|C:\\neo\\validator-2.wallet.json|https://signer.example.test/validator-2",
        committee_key_values[0], committee_key_values[1]
    );
    let request = PrivateNetworkDeploymentRequest {
        plan: plan.clone(),
        nodes: created.clone(),
        plugin_states: BTreeMap::new(),
        committee: CommitteeRoster::from_public_keys_and_references(&committee_keys, &signer_refs)
            .unwrap(),
        output_dir: temp_dir.path().join("private-networks"),
        node_root_dir: temp_dir.path().join("nodes"),
    };

    let export = PrivateNetworkDeploymentExporter::write(request).unwrap();

    let first_config_sha256 = manifest_assertions::assert_manifest(&export);
    documents::assert_documents(&export);
    scripts::assert_unix_scripts(&export, &first_config_sha256);
    scripts::assert_windows_scripts(&export, &first_config_sha256);
    configs::assert_managed_configs(temp_dir.path(), &created);
}
