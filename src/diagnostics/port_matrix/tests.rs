use super::*;
use crate::{
    diagnostics::{DiagnosticCheck, NodeDiagnostics},
    types::{NodeType, StorageEngine},
};

#[test]
fn port_matrix_prioritizes_critical_running_nodes() {
    let nodes = vec![
        node(
            "stopped",
            "Stopped",
            NodeStatus::Stopped,
            Network::Private,
            30332,
        ),
        node(
            "running",
            "Running",
            NodeStatus::Running,
            Network::Mainnet,
            10332,
        ),
    ];
    let diagnostics = fleet(vec![
        diagnostic("stopped", CheckSeverity::Critical),
        diagnostic("running", CheckSeverity::Critical),
    ]);

    let rows = filter_port_matrix_rows(&nodes, &diagnostics, &PortMatrixFilter::default());

    assert_eq!(rows[0].node_id, "running");
    assert_eq!(rows[1].node_id, "stopped");
}

#[test]
fn port_matrix_filters_status_network_health_and_query() {
    let nodes = vec![
        node(
            "main-rpc",
            "Main RPC",
            NodeStatus::Running,
            Network::Mainnet,
            10332,
        ),
        node(
            "test-rpc",
            "Test RPC",
            NodeStatus::Stopped,
            Network::Testnet,
            20332,
        ),
    ];
    let diagnostics = fleet(vec![
        diagnostic("main-rpc", CheckSeverity::Critical),
        diagnostic("test-rpc", CheckSeverity::Pass),
    ]);

    let rows = filter_port_matrix_rows(
        &nodes,
        &diagnostics,
        &PortMatrixFilter::new(
            Some(NodeStatus::Running),
            Some(Network::Mainnet),
            Some(CheckSeverity::Critical),
            "10332",
        ),
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].node_id, "main-rpc");
}

fn node(id: &str, name: &str, status: NodeStatus, network: Network, rpc_port: u16) -> NodeConfig {
    NodeConfig {
        id: id.to_string(),
        name: name.to_string(),
        node_type: NodeType::NeoRs,
        network,
        binary_path: "/usr/local/bin/neo-node".into(),
        args: Vec::new(),
        runtime_version: "latest".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port,
        p2p_port: rpc_port + 1,
        ws_port: Some(rpc_port + 2),
        status,
        pid: None,
    }
}

fn fleet(nodes: Vec<NodeDiagnostics>) -> FleetDiagnostics {
    FleetDiagnostics {
        score: 0,
        ready_nodes: 0,
        warning_count: 0,
        critical_count: 0,
        nodes,
    }
}

fn diagnostic(node_id: &str, severity: CheckSeverity) -> NodeDiagnostics {
    NodeDiagnostics {
        node_id: node_id.to_string(),
        node_name: node_id.to_string(),
        score: 0,
        checks: vec![DiagnosticCheck {
            severity,
            title: "Network",
            detail: "port state".to_string(),
        }],
    }
}
