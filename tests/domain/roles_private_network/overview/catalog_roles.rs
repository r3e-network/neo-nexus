use crate::*;

#[test]
fn plugin_catalog_contains_neo_cli_operational_plugins() {
    let catalog = PluginCatalog;
    let plugins = catalog.for_node_type(NodeType::NeoCli);
    let ids: Vec<PluginId> = plugins.iter().map(|plugin| plugin.id).collect();

    assert!(ids.contains(&PluginId::RpcServer));
    assert!(ids.contains(&PluginId::ApplicationLogs));
    assert!(ids.contains(&PluginId::StateService));
    assert!(ids.contains(&PluginId::DBFTPlugin));
}

#[test]
fn node_type_parses_neo_rs_runtime() {
    assert_eq!(NodeType::from_str("neo-rs").unwrap(), NodeType::NeoRs);
    assert_eq!(NodeType::NeoRs.to_string(), "neo-rs");
}

#[test]
fn neo_rs_has_no_neo_cli_plugin_catalog_entries() {
    let catalog = PluginCatalog;
    assert!(catalog.for_node_type(NodeType::NeoRs).is_empty());
}

#[test]
fn role_planner_enables_neo_cli_indexer_plugins_and_storage() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-cli indexer", NodeType::NeoCli);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let plan = RolePlanner::plan(&node, NodeRole::Indexer);

    assert_eq!(plan.role, NodeRole::Indexer);
    assert_eq!(
        planned_plugin_enabled(&plan, PluginId::RpcServer),
        Some(true)
    );
    assert_eq!(
        planned_plugin_enabled(&plan, PluginId::ApplicationLogs),
        Some(true)
    );
    assert_eq!(
        planned_plugin_enabled(&plan, PluginId::TokensTracker),
        Some(true)
    );
    assert_eq!(
        planned_plugin_enabled(&plan, PluginId::StateService),
        Some(true)
    );
    assert_eq!(
        planned_plugin_enabled(&plan, PluginId::RocksDbStore),
        Some(true)
    );
    assert_eq!(
        planned_plugin_enabled(&plan, PluginId::LevelDbStore),
        Some(false)
    );
    assert_eq!(
        planned_plugin_enabled(&plan, PluginId::DBFTPlugin),
        Some(false)
    );
}

#[test]
fn role_planner_keeps_neo_rs_runtime_managed_by_config() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-rs api", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let plan = RolePlanner::plan(&node, NodeRole::RpcApi);

    assert_eq!(plan.node_type, NodeType::NeoRs);
    assert!(plan.plugin_changes.is_empty());
    assert!(plan.notes.iter().any(|note| note.contains("TOML")));
}
