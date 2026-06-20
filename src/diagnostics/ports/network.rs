use crate::{diagnostics::DiagnosticCheck, types::NodeConfig};

use super::{
    checks::{critical_checks, pass_check},
    node_ports::NodePorts,
};

const NETWORK_TITLE: &str = "Network";
const NETWORK_PASS_DETAIL: &str = "RPC, P2P, and WebSocket ports do not collide with other nodes.";

pub(in crate::diagnostics) fn port_checks(
    node: &NodeConfig,
    all_nodes: &[NodeConfig],
) -> Vec<DiagnosticCheck> {
    let conflicts = network_port_conflicts(node, all_nodes);
    if conflicts.is_empty() {
        pass_check(NETWORK_TITLE, NETWORK_PASS_DETAIL)
    } else {
        critical_checks(NETWORK_TITLE, conflicts)
    }
}

fn network_port_conflicts(node: &NodeConfig, all_nodes: &[NodeConfig]) -> Vec<String> {
    node.ports()
        .into_iter()
        .flat_map(|(label, port)| {
            all_nodes
                .iter()
                .filter(move |other| other.id != node.id)
                .flat_map(move |other| node_conflicts(label, port, other))
        })
        .collect()
}

fn node_conflicts(label: &'static str, port: u16, other: &NodeConfig) -> Vec<String> {
    other
        .ports()
        .into_iter()
        .filter_map(|(other_label, other_port)| {
            if other_port == port {
                Some(format!(
                    "{label} {port} overlaps with {} {other_label}",
                    other.name
                ))
            } else {
                None
            }
        })
        .collect()
}
