use crate::*;

/// A node name is a join key, so the workspace refuses a second node with one
/// that is taken.
///
/// `nodes.name` had no UNIQUE constraint while the private-network launch-pack
/// exporter selects members **by name** — uniqueness was checked at plan time
/// and never enforced by the schema, so two nodes sharing a name silently wrote
/// one member's config twice. Every CLI command that takes `<node-name>` had
/// the same ambiguity.
#[test]
fn a_second_node_cannot_take_a_name_that_is_already_used() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    repo.create_node(node_named("duplicate validator", 10332))
        .expect("the first node takes the name");

    let refused = repo.create_node(node_named("duplicate validator", 20332));
    assert!(
        refused.is_err(),
        "the workspace accepted two nodes with one name"
    );
}

/// Exported config paths are keyed by **id**, not by name.
///
/// That is what made duplicate names survivable before the constraint existed,
/// and it is still the property worth holding: a name is operator text and can
/// be edited, so a path derived from one would move when it changed.
#[test]
fn workspace_config_exporter_keys_files_by_node_id() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let first = repo.create_node(node_named("validator-a", 10332)).unwrap();
    let second = repo.create_node(node_named("validator-b", 20332)).unwrap();

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
    for id in [first.id.as_str(), second.id.as_str()] {
        assert!(
            paths
                .iter()
                .any(|path| path.components().any(|component| component
                    .as_os_str()
                    .to_string_lossy()
                    .as_ref()
                    == id)),
            "no exported path is keyed by {id}"
        );
    }
}

fn node_named(name: &str, rpc_port: u16) -> NewNode {
    NewNode {
        name: name.to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/usr/local/bin/neo-node"),
        args: Vec::new(),
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port,
        p2p_port: rpc_port + 1,
        ws_port: Some(rpc_port + 2),
    }
}
