use crate::types::{Network, NodeConfig, NodeStatus};

use super::{CheckSeverity, FleetDiagnostics, NodeDiagnostics};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PortMatrixFilter {
    pub status: Option<NodeStatus>,
    pub network: Option<Network>,
    pub health: Option<CheckSeverity>,
    pub query: String,
}

impl PortMatrixFilter {
    pub fn new(
        status: Option<NodeStatus>,
        network: Option<Network>,
        health: Option<CheckSeverity>,
        query: impl Into<String>,
    ) -> Self {
        Self {
            status,
            network,
            health,
            query: query.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortMatrixRow {
    pub node_id: String,
    pub node_name: String,
    pub network: Network,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub ws_port: Option<u16>,
    pub status: NodeStatus,
    pub health: CheckSeverity,
}

pub fn filter_port_matrix_rows(
    nodes: &[NodeConfig],
    diagnostics: &FleetDiagnostics,
    filter: &PortMatrixFilter,
) -> Vec<PortMatrixRow> {
    let query = filter.query.trim().to_lowercase();
    let mut rows = nodes
        .iter()
        .map(|node| row_for_node(node, diagnostics))
        .filter(|row| filter.status.is_none_or(|status| row.status == status))
        .filter(|row| filter.network.is_none_or(|network| row.network == network))
        .filter(|row| filter.health.is_none_or(|health| row.health == health))
        .filter(|row| query.is_empty() || row_matches(row, &query))
        .collect::<Vec<_>>();
    rows.sort_by(row_order);
    rows
}

fn row_for_node(node: &NodeConfig, diagnostics: &FleetDiagnostics) -> PortMatrixRow {
    let health = diagnostics
        .nodes
        .iter()
        .find(|diagnostic| diagnostic.node_id == node.id)
        .map_or(CheckSeverity::Info, network_severity);

    PortMatrixRow {
        node_id: node.id.clone(),
        node_name: node.name.clone(),
        network: node.network,
        rpc_port: node.rpc_port,
        p2p_port: node.p2p_port,
        ws_port: node.ws_port,
        status: node.status,
        health,
    }
}

fn network_severity(diagnostic: &NodeDiagnostics) -> CheckSeverity {
    if diagnostic
        .checks
        .iter()
        .any(|check| check.title == "Network" && check.severity == CheckSeverity::Critical)
    {
        CheckSeverity::Critical
    } else {
        CheckSeverity::Pass
    }
}

fn row_order(left: &PortMatrixRow, right: &PortMatrixRow) -> std::cmp::Ordering {
    right
        .health
        .cmp(&left.health)
        .then_with(|| status_rank(left.status).cmp(&status_rank(right.status)))
        .then_with(|| left.network.to_string().cmp(&right.network.to_string()))
        .then_with(|| left.node_name.cmp(&right.node_name))
}

fn status_rank(status: NodeStatus) -> u8 {
    match status {
        NodeStatus::Running => 0,
        NodeStatus::Starting => 1,
        NodeStatus::Error => 2,
        NodeStatus::Stopped => 3,
    }
}

fn row_matches(row: &PortMatrixRow, query: &str) -> bool {
    text_matches(&row.node_id, query)
        || text_matches(&row.node_name, query)
        || text_matches(&row.network.to_string(), query)
        || text_matches(&row.rpc_port.to_string(), query)
        || text_matches(&row.p2p_port.to_string(), query)
        || row
            .ws_port
            .is_some_and(|port| text_matches(&port.to_string(), query))
        || text_matches(row.status.label(), query)
        || text_matches(row.health.label(), query)
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
#[path = "../../tests/unit/diagnostics/port_matrix/tests.rs"]
mod tests;
