use crate::*;

#[test]
fn workspace_config_exporter_keeps_duplicate_node_names_separate() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let first = repo
        .create_node(NewNode {
            name: "duplicate validator".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: PathBuf::from("/usr/local/bin/neo-node-a"),
            args: Vec::new(),
            runtime_version: "v0.8.0".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: Some(10334),
        })
        .unwrap();
    let second = repo
        .create_node(NewNode {
            name: "duplicate validator".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: PathBuf::from("/usr/local/bin/neo-node-b"),
            args: Vec::new(),
            runtime_version: "v0.8.1".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: Some(20334),
        })
        .unwrap();
    let nodes_with_plugins = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .map(|node| (node, Vec::new()))
        .collect::<Vec<_>>();

    let export = WorkspaceConfigExporter::write_at(
        temp_dir.path().join("bulk-configs"),
        repo.db_path(),
        &nodes_with_plugins,
        "test",
        1_800_000_000,
    )
    .unwrap();

    assert_eq!(export.report.node_count, 2);
    assert_eq!(export.report.exported_file_count, 2);
    assert!(export.text_path.is_file());
    assert!(export.json_path.is_file());
    let paths = export
        .report
        .nodes
        .iter()
        .map(|node| PathBuf::from(&node.path))
        .collect::<Vec<_>>();
    assert_ne!(paths[0], paths[1]);
    assert!(paths.iter().all(|path| path.is_file()));
    assert!(paths
        .iter()
        .any(|path| path.components().any(|component| component
            .as_os_str()
            .to_string_lossy()
            .as_ref()
            == first.id.as_str())));
    assert!(paths
        .iter()
        .any(|path| path.components().any(|component| component
            .as_os_str()
            .to_string_lossy()
            .as_ref()
            == second.id.as_str())));
}
