use super::*;

#[test]
fn plugin_filter_limits_catalog_selection_and_pages() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let node = repository.create_node(plugin_node("Validator Plugins", 31_332, 31_333))?;
    repository.set_plugin_enabled(&node.id, PluginId::RpcServer, true)?;
    repository.set_plugin_enabled(&node.id, PluginId::RestServer, false)?;
    let mut app = NeoNexusApp::new(repository);
    let node = app
        .selected_node()
        .cloned()
        .expect("node should be selected");

    app.plugin_query = "api".to_string();
    app.plugin_enabled_filter = Some(true);
    app.plugin_category_filter = Some(PluginCategory::Api);

    let visible = app.filtered_plugins_for_node(&node);
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, PluginId::RpcServer);

    app.selected_plugin = Some(PluginId::ApplicationLogs);
    app.plugin_page = 4;
    app.ensure_valid_plugin_selection(&node);
    assert_eq!(app.selected_plugin, Some(PluginId::RpcServer));
    assert_eq!(app.plugin_page, 0);

    Ok(())
}

fn plugin_node(name: &str, rpc_port: u16, p2p_port: u16) -> NewNode {
    NewNode {
        name: name.to_string(),
        node_type: NodeType::NeoCli,
        network: Network::Testnet,
        binary_path: PathBuf::from("neo-cli"),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::LevelDb,
        rpc_port,
        p2p_port,
        ws_port: None,
    }
}
