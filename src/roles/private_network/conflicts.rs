use crate::types::NodeConfig;

use super::plan::{PrivateNetworkNodePlan, PrivateNetworkPlan};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateNetworkConflict {
    pub planned_node: String,
    pub field: &'static str,
    pub detail: String,
}

pub(super) fn detect_conflicts(
    plan: &PrivateNetworkPlan,
    existing_nodes: &[NodeConfig],
) -> Vec<PrivateNetworkConflict> {
    let mut conflicts = Vec::new();

    for planned in &plan.nodes {
        for existing in existing_nodes {
            if existing.name == planned.name {
                conflicts.push(PrivateNetworkConflict {
                    planned_node: planned.name.clone(),
                    field: "name",
                    detail: format!("{} already exists", planned.name),
                });
            }

            for (planned_label, planned_port) in planned_ports(planned) {
                for (existing_label, existing_port) in existing_ports(existing) {
                    if planned_port == existing_port {
                        conflicts.push(PrivateNetworkConflict {
                            planned_node: planned.name.clone(),
                            field: planned_label,
                            detail: format!(
                                "{} {planned_label} port {planned_port} overlaps with {} {existing_label}",
                                planned.name, existing.name
                            ),
                        });
                    }
                }
            }
        }
    }

    conflicts
}

fn planned_ports(node: &PrivateNetworkNodePlan) -> Vec<(&'static str, u16)> {
    let mut ports = vec![("RPC", node.rpc_port), ("P2P", node.p2p_port)];
    if let Some(port) = node.ws_port {
        ports.push(("WS", port));
    }
    ports
}

fn existing_ports(node: &NodeConfig) -> Vec<(&'static str, u16)> {
    let mut ports = vec![("RPC", node.rpc_port), ("P2P", node.p2p_port)];
    if let Some(port) = node.ws_port {
        ports.push(("WS", port));
    }
    ports
}
