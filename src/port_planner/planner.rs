use std::collections::BTreeSet;

use anyhow::Result;

use crate::types::NodeConfig;

use super::{is_localhost_tcp_port_available, PortAssignment};

pub const DEFAULT_RPC_PORT: u16 = 10_332;
const PORT_BLOCK_STEP: u16 = 10;

pub fn plan_available_node_ports(
    nodes: &[NodeConfig],
    current_node_id: Option<&str>,
    preferred_rpc_port: u16,
    include_ws: bool,
) -> Result<PortAssignment> {
    plan_available_node_ports_with(
        nodes,
        current_node_id,
        preferred_rpc_port,
        include_ws,
        is_localhost_tcp_port_available,
    )
}

pub fn plan_available_node_ports_with<F>(
    nodes: &[NodeConfig],
    current_node_id: Option<&str>,
    preferred_rpc_port: u16,
    include_ws: bool,
    mut is_port_available: F,
) -> Result<PortAssignment>
where
    F: FnMut(u16) -> bool,
{
    let reserved_ports = reserved_node_ports(nodes, current_node_id);
    let start = normalize_start_port(preferred_rpc_port);
    let max_base = u16::MAX - if include_ws { 2 } else { 1 };
    let mut visited = BTreeSet::new();

    for base in block_candidates(start, max_base) {
        if visited.insert(base) {
            if let Some(assignment) =
                available_assignment(base, include_ws, &reserved_ports, &mut is_port_available)
            {
                return Ok(assignment);
            }
        }
    }

    for base in 1..=max_base {
        if visited.insert(base) {
            if let Some(assignment) =
                available_assignment(base, include_ws, &reserved_ports, &mut is_port_available)
            {
                return Ok(assignment);
            }
        }
    }

    anyhow::bail!("no available local port block could be found")
}

fn reserved_node_ports(nodes: &[NodeConfig], current_node_id: Option<&str>) -> BTreeSet<u16> {
    let mut ports = BTreeSet::new();
    for node in nodes {
        if current_node_id.is_some_and(|current_id| current_id == node.id) {
            continue;
        }
        ports.insert(node.rpc_port);
        ports.insert(node.p2p_port);
        if let Some(ws_port) = node.ws_port {
            ports.insert(ws_port);
        }
    }
    ports
}

fn normalize_start_port(preferred_rpc_port: u16) -> u16 {
    if preferred_rpc_port == 0 {
        DEFAULT_RPC_PORT
    } else {
        preferred_rpc_port
    }
}

fn block_candidates(start: u16, max_base: u16) -> impl Iterator<Item = u16> {
    (start..=max_base).step_by(PORT_BLOCK_STEP as usize)
}

fn available_assignment<F>(
    base: u16,
    include_ws: bool,
    reserved_ports: &BTreeSet<u16>,
    is_port_available: &mut F,
) -> Option<PortAssignment>
where
    F: FnMut(u16) -> bool,
{
    let assignment = PortAssignment {
        rpc_port: base,
        p2p_port: base.checked_add(1)?,
        ws_port: include_ws.then(|| base.checked_add(2)).flatten(),
    };
    let ports = assignment.ports();
    let unique_ports = ports.iter().map(|(_, port)| *port).collect::<BTreeSet<_>>();
    if unique_ports.len() != ports.len() {
        return None;
    }
    if ports
        .iter()
        .any(|(_, port)| reserved_ports.contains(port) || !is_port_available(*port))
    {
        return None;
    }
    Some(assignment)
}
