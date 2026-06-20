use crate::*;

#[test]
fn private_network_planner_generates_deterministic_neo_rs_topology() {
    let plan = PrivateNetworkPlanner::plan(PrivateNetworkTemplate::SevenNodeLab, NodeType::NeoRs);

    assert_eq!(plan.nodes.len(), 7);
    assert_eq!(plan.consensus_count(), 4);
    assert_eq!(plan.nodes[0].name, "neo-rs-validator-1");
    assert_eq!(plan.nodes[0].role, NodeRole::Consensus);
    assert_eq!(plan.nodes[0].network, Network::Private);
    assert_eq!(plan.nodes[0].rpc_port, 30332);
    assert_eq!(plan.nodes[0].p2p_port, 30333);
    assert_eq!(plan.nodes[0].ws_port, Some(30334));
    assert_eq!(plan.nodes[4].role, NodeRole::RpcApi);
    assert_eq!(plan.nodes[4].rpc_port, 30372);
    assert!(plan
        .nodes
        .iter()
        .all(|node| node.storage_engine == StorageEngine::RocksDb));
}

#[test]
fn private_network_plan_materializes_nodes_from_template_without_args() {
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
    let plan =
        PrivateNetworkPlanner::plan(PrivateNetworkTemplate::SingleValidator, NodeType::NeoRs);

    let materialized = plan.to_new_nodes(&template).unwrap();

    assert_eq!(materialized.len(), 1);
    assert_eq!(materialized[0].name, "neo-rs-validator-1");
    assert_eq!(materialized[0].node_type, NodeType::NeoRs);
    assert_eq!(materialized[0].network, Network::Private);
    assert_eq!(
        materialized[0].binary_path,
        PathBuf::from("/opt/neo-rs/neo-node")
    );
    assert!(materialized[0].args.is_empty());
    assert_eq!(materialized[0].runtime_version, "v0.8.0");
    assert_eq!(materialized[0].rpc_port, 30332);
}

#[test]
fn private_network_plan_detects_existing_name_and_port_conflicts() {
    let repo = create_repo();
    let existing_id = repo
        .create_node(NewNode {
            name: "neo-rs-validator-1".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Private,
            binary_path: PathBuf::from("/opt/neo-rs/neo-node"),
            args: Vec::new(),
            runtime_version: "v0.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 30332,
            p2p_port: 30333,
            ws_port: Some(30334),
        })
        .unwrap()
        .id;
    let existing = repo.list_nodes().unwrap();
    assert!(existing.iter().any(|node| node.id == existing_id));
    let plan =
        PrivateNetworkPlanner::plan(PrivateNetworkTemplate::SingleValidator, NodeType::NeoRs);

    let conflicts = plan.conflicts_with(&existing);

    assert!(conflicts.iter().any(|conflict| conflict.field == "name"));
    assert!(conflicts
        .iter()
        .any(|conflict| conflict.detail.contains("RPC port 30332")));
}

#[test]
fn repository_materializes_private_network_nodes_with_role_plugins() {
    let repo = create_repo();
    let template_id = create_node(&repo, "neo-cli template", NodeType::NeoCli);
    let template = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == template_id)
        .unwrap();
    let plan =
        PrivateNetworkPlanner::plan(PrivateNetworkTemplate::SingleValidator, NodeType::NeoCli);
    let inputs = plan.to_new_nodes(&template).unwrap();
    let inputs_with_plugins: Vec<_> = plan
        .nodes
        .iter()
        .zip(inputs)
        .map(|(planned, input)| {
            let role_plan =
                RolePlanner::plan_for(input.node_type, input.storage_engine, planned.role);
            let plugins = role_plan
                .plugin_changes
                .into_iter()
                .map(|change| PluginState {
                    plugin_id: change.plugin_id,
                    enabled: change.enabled,
                })
                .collect();
            (input, plugins)
        })
        .collect();

    let created = repo.create_nodes_with_plugins(inputs_with_plugins).unwrap();

    assert_eq!(created.len(), 1);
    assert_eq!(created[0].name, "neo-cli-validator-1");
    assert_eq!(created[0].network, Network::Private);
    let plugins = repo.list_plugin_states(&created[0].id).unwrap();
    assert!(plugins
        .iter()
        .any(|plugin| plugin.plugin_id == PluginId::DBFTPlugin && plugin.enabled));
    assert!(plugins
        .iter()
        .any(|plugin| plugin.plugin_id == PluginId::RpcServer && !plugin.enabled));
    assert!(plugins
        .iter()
        .any(|plugin| plugin.plugin_id == PluginId::RocksDbStore && plugin.enabled));
}
