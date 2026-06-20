use super::*;

#[test]
fn persists_plugin_installation_inventory_and_deletes_with_node() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "managed".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: "/usr/local/bin/neo-cli".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();
    let installation = PluginInstallation {
        node_id: node.id.clone(),
        plugin_id: PluginId::RpcServer,
        installed_path: temp_dir
            .path()
            .join("nodes")
            .join(&node.id)
            .join("Plugins/RpcServer"),
        manifest_path: temp_dir
            .path()
            .join("nodes")
            .join(&node.id)
            .join("Plugins/RpcServer/.neonexus/manifest.json"),
        source_path: PathBuf::from("/tmp/rpc-server.zip"),
        sha256: "b".repeat(64),
        package_bytes: 1024,
        installed_files: 4,
        expanded_bytes: 2048,
        installed_at_unix: 1_800_000_100,
    };

    repository
        .upsert_plugin_installation(&installation)
        .unwrap();
    let persisted = repository.list_plugin_installations(&node.id).unwrap();

    assert_eq!(persisted, vec![installation]);

    repository.delete_node(&node.id).unwrap();

    assert!(repository
        .list_plugin_installations(&node.id)
        .unwrap()
        .is_empty());
}
