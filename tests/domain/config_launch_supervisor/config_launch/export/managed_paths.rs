use crate::*;

#[test]
fn config_exporter_uses_runtime_specific_managed_paths() {
    let repo = create_repo();
    let neo_cli_id = create_node(&repo, "neo-cli managed", NodeType::NeoCli);
    let neo_go_id = create_node(&repo, "neo-go managed", NodeType::NeoGo);
    let neo_rs_id = create_node(&repo, "neo-rs managed", NodeType::NeoRs);
    let nodes = repo.list_nodes().unwrap();
    let neo_cli = nodes.iter().find(|node| node.id == neo_cli_id).unwrap();
    let neo_go = nodes.iter().find(|node| node.id == neo_go_id).unwrap();
    let neo_rs = nodes.iter().find(|node| node.id == neo_rs_id).unwrap();
    let root = PathBuf::from("/tmp/neonexus-node");

    assert_eq!(
        ConfigExporter::managed_target_path(&root, neo_cli),
        PathBuf::from("/tmp/neonexus-node/config.json")
    );
    assert!(ConfigExporter::managed_target_path(&root, neo_go)
        .ends_with("config/neo-go-managed-neo-go-config.yml"));
    assert!(ConfigExporter::managed_target_path(&root, neo_rs)
        .ends_with("config/neo-rs-managed-neo-rs-config.toml"));
}

#[test]
fn config_exporter_writes_managed_config_to_exact_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node_id = create_node(&repo, "neo-rs managed", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let path = temp_dir
        .path()
        .join("nodes")
        .join(&node.id)
        .join("config")
        .join("managed.toml");

    let export = ConfigExporter::write_node_config_to_path(&path, &node, &[]).unwrap();

    assert_eq!(export.path, path);
    assert!(export.path.is_file());
    let text = std::fs::read_to_string(export.path).unwrap();
    assert!(text.contains("[network]"));
    assert!(text.contains("network_type = \"testnet\""));
}
