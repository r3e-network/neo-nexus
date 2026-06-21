use std::collections::BTreeSet;

use crate::{diagnostics::DiagnosticCheck, types::NodeConfig};

use super::{
    checks::{critical_checks, pass_check},
    node_ports::NodePorts,
};

const LAUNCH_PORTS_TITLE: &str = "Launch ports";
const LAUNCH_PORTS_PASS_DETAIL: &str =
    "No active running or starting node or localhost listener uses this node's ports.";

pub(in crate::diagnostics) fn launch_port_checks<F>(
    node: &NodeConfig,
    all_nodes: &[NodeConfig],
    mut is_port_available: F,
) -> Vec<DiagnosticCheck>
where
    F: FnMut(u16) -> bool,
{
    let node_ports = node.ports();
    let mut conflicts = Vec::new();
    let active_conflict_ports =
        active_node_port_conflicts(node, all_nodes, &node_ports, &mut conflicts);

    if !is_active_node(node) {
        conflicts.extend(localhost_listener_conflicts(
            node_ports,
            &active_conflict_ports,
            &mut is_port_available,
        ));
    }

    if conflicts.is_empty() {
        pass_check(LAUNCH_PORTS_TITLE, LAUNCH_PORTS_PASS_DETAIL)
    } else {
        critical_checks(LAUNCH_PORTS_TITLE, conflicts)
    }
}

fn active_node_port_conflicts(
    node: &NodeConfig,
    all_nodes: &[NodeConfig],
    node_ports: &[(&'static str, u16)],
    conflicts: &mut Vec<String>,
) -> BTreeSet<u16> {
    let mut active_conflict_ports = BTreeSet::new();
    for (label, port) in node_ports {
        for other in all_nodes
            .iter()
            .filter(|other| other.id != node.id && is_active_node(other))
        {
            for (other_label, other_port) in other.ports() {
                if other_port == *port {
                    active_conflict_ports.insert(*port);
                    conflicts.push(format!(
                        "{label} port {port} overlaps with active {} {other_label}.",
                        other.name
                    ));
                }
            }
        }
    }
    active_conflict_ports
}

fn localhost_listener_conflicts<F>(
    node_ports: Vec<(&'static str, u16)>,
    active_conflict_ports: &BTreeSet<u16>,
    is_port_available: &mut F,
) -> Vec<String>
where
    F: FnMut(u16) -> bool,
{
    node_ports
        .into_iter()
        .filter(|(_, port)| !active_conflict_ports.contains(port))
        .filter_map(|(label, port)| {
            if is_port_available(port) {
                None
            } else {
                Some(format!(
                    "{label} port {port} is already listening on localhost (127.0.0.1 or ::1); use Node Studio Fix Ports or stop the occupying process."
                ))
            }
        })
        .collect()
}

fn is_active_node(node: &NodeConfig) -> bool {
    node.status.is_active()
}
