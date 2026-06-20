use super::*;

mod ports;
mod processes;
#[cfg(unix)]
mod upgrades;

fn neo_rs_app_node(name: &str, rpc_port: u16, p2p_port: u16, ws_port: Option<u16>) -> NewNode {
    NewNode {
        name: name.to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("neo-node"),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port,
        p2p_port,
        ws_port,
    }
}

#[test]
fn node_inventory_filter_limits_visible_selection_set() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let alpha = repository.create_node(neo_rs_app_node("RPC Alpha", 33_332, 33_333, None))?;
    repository.create_node(neo_rs_app_node("RPC Beta", 34_332, 34_333, None))?;
    let seed = repository.create_node(neo_rs_app_node("Seed Beta", 35_332, 35_333, None))?;
    let mut app = NeoNexusApp::new(repository);
    app.repository
        .update_node_status(&seed.id, NodeStatus::Running, Some(44))?;
    app.reload_nodes();
    app.node_query = "beta".to_string();
    app.node_status_filter = Some(NodeStatus::Running);

    let visible = app.filtered_inventory_nodes();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, seed.id);

    app.selected_node = Some(alpha.id);
    assert_eq!(app.selected_filtered_node_index(&visible), None);
    app.selected_node = Some(seed.id);
    assert_eq!(app.selected_filtered_node_index(&visible), Some(0));

    app.node_page = 9;
    app.ensure_valid_selection();
    assert_eq!(app.node_page, 0);

    Ok(())
}

#[test]
fn overview_fleet_page_clamps_with_inventory_filter() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    repository.create_node(neo_rs_app_node("RPC Alpha", 36_332, 36_333, None))?;
    let beta = repository.create_node(neo_rs_app_node("RPC Beta", 37_332, 37_333, None))?;
    let mut app = NeoNexusApp::new(repository);
    app.repository
        .update_node_status(&beta.id, NodeStatus::Running, Some(45))?;
    app.reload_nodes();

    app.node_query = "beta".to_string();
    app.node_status_filter = Some(NodeStatus::Running);
    app.overview_fleet_page = 12;

    let visible = app.filtered_inventory_nodes();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, beta.id);

    app.ensure_valid_selection();
    assert_eq!(app.overview_fleet_page, 0);

    Ok(())
}
