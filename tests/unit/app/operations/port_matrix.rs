use super::super::*;
use crate::diagnostics::{
    CheckSeverity, DiagnosticCheck, DiagnosticResolution, FleetDiagnostics, NodeDiagnostics,
    PortMatrixRow,
};

#[test]
fn port_matrix_filters_rows_and_clamps_page() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let main = repository.create_node(port_node("main-rpc", Network::Mainnet, 10332))?;
    repository.create_node(port_node("test-rpc", Network::Testnet, 20332))?;
    let mut app = NeoNexusApp::new(repository);
    app.repository
        .update_node_status(&main.id, NodeStatus::Running, Some(42))?;
    app.reload_nodes();
    let main_id = app.fleet
        .nodes
        .iter()
        .find(|node| node.name == "main-rpc")
        .map(|node| node.id.clone())
        .ok_or_else(|| anyhow::anyhow!("main node should exist"))?;
    let test_id = app.fleet
        .nodes
        .iter()
        .find(|node| node.name == "test-rpc")
        .map(|node| node.id.clone())
        .ok_or_else(|| anyhow::anyhow!("test node should exist"))?;
    let diagnostics = FleetDiagnostics {
        score: 50,
        ready_nodes: 0,
        warning_count: 0,
        critical_count: 1,
        nodes: vec![
            diagnostic(&main_id, "main-rpc", CheckSeverity::Critical),
            diagnostic(&test_id, "test-rpc", CheckSeverity::Pass),
        ],
    };

    app.operations_ui.port_matrix_status_filter = Some(NodeStatus::Running);
    app.operations_ui.port_matrix_network_filter = Some(Network::Mainnet);
    app.operations_ui.port_matrix_health_filter = Some(CheckSeverity::Critical);
    app.operations_ui.port_matrix_query = "10332".to_string();
    app.operations_ui.port_matrix_page = 7;

    let visible = app.filtered_port_matrix_rows(&diagnostics);
    assert_eq!(visible.len(), 1);
    assert_eq!(
        visible[0],
        PortMatrixRow {
            node_id: main_id,
            node_name: "main-rpc".to_string(),
            network: Network::Mainnet,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: Some(10334),
            status: NodeStatus::Running,
            health: CheckSeverity::Critical,
        }
    );

    app.clamp_port_matrix_page(&diagnostics);
    assert_eq!(app.operations_ui.port_matrix_page, 0);

    Ok(())
}

#[test]
fn port_matrix_focuses_blocked_ports_and_clears_filters() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let main = repository.create_node(port_node("main-rpc", Network::Mainnet, 10332))?;
    repository.create_node(port_node("test-rpc", Network::Testnet, 20332))?;
    let mut app = NeoNexusApp::new(repository);
    app.repository
        .update_node_status(&main.id, NodeStatus::Running, Some(42))?;
    app.reload_nodes();
    let main_id = app.fleet
        .nodes
        .iter()
        .find(|node| node.name == "main-rpc")
        .map(|node| node.id.clone())
        .ok_or_else(|| anyhow::anyhow!("main node should exist"))?;
    let test_id = app.fleet
        .nodes
        .iter()
        .find(|node| node.name == "test-rpc")
        .map(|node| node.id.clone())
        .ok_or_else(|| anyhow::anyhow!("test node should exist"))?;
    let diagnostics = FleetDiagnostics {
        score: 50,
        ready_nodes: 0,
        warning_count: 0,
        critical_count: 1,
        nodes: vec![
            diagnostic(&main_id, "main-rpc", CheckSeverity::Critical),
            diagnostic(&test_id, "test-rpc", CheckSeverity::Pass),
        ],
    };

    app.operations_ui.port_matrix_status_filter = Some(NodeStatus::Stopped);
    app.operations_ui.port_matrix_network_filter = Some(Network::Testnet);
    app.operations_ui.port_matrix_health_filter = Some(CheckSeverity::Pass);
    app.operations_ui.port_matrix_query = "20332".to_string();
    app.operations_ui.port_matrix_page = 3;

    app.focus_blocked_ports(&diagnostics);

    assert_eq!(app.operations_ui.port_matrix_status_filter, None);
    assert_eq!(app.operations_ui.port_matrix_network_filter, None);
    assert_eq!(app.operations_ui.port_matrix_health_filter, Some(CheckSeverity::Critical));
    assert!(app.operations_ui.port_matrix_query.is_empty());
    assert_eq!(app.operations_ui.port_matrix_page, 0);
    assert_eq!(app.fleet.selected_node.as_deref(), Some(main_id.as_str()));
    let blocked_rows = app.filtered_port_matrix_rows(&diagnostics);
    assert_eq!(blocked_rows.len(), 1);
    assert_eq!(blocked_rows[0].node_id, main_id);
    assert!(app.has_active_port_matrix_filter());

    app.clear_port_matrix_filters(&diagnostics);

    assert!(!app.has_active_port_matrix_filter());
    assert_eq!(app.filtered_port_matrix_rows(&diagnostics).len(), 2);
    assert!(app
        .selected_visible_port_matrix_row(&app.filtered_port_matrix_rows(&diagnostics))
        .is_some());

    Ok(())
}

fn port_node(name: &str, network: Network, rpc_port: u16) -> NewNode {
    NewNode {
        name: name.to_string(),
        node_type: NodeType::NeoRs,
        network,
        binary_path: PathBuf::from("/usr/local/bin/neo-node"),
        args: Vec::new(),
        runtime_version: "latest".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port,
        p2p_port: rpc_port + 1,
        ws_port: Some(rpc_port + 2),
    }
}

fn diagnostic(node_id: &str, node_name: &str, severity: CheckSeverity) -> NodeDiagnostics {
    NodeDiagnostics {
        node_id: node_id.to_string(),
        node_name: node_name.to_string(),
        score: 0,
        checks: vec![DiagnosticCheck::new(
            severity,
            "Network",
            "port state",
            DiagnosticResolution::NodeStudio,
        )],
    }
}
