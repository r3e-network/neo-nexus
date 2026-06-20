use std::collections::BTreeMap;

use crate::{catalog::PluginState, types::NodeConfig};

use super::{
    checks::{binary_checks, config_checks, plugin_checks, status_check, version_check},
    ports::port_checks,
    FleetDiagnostics, NodeDiagnostics,
};

pub fn evaluate_fleet(
    nodes: &[NodeConfig],
    plugin_states: &BTreeMap<String, Vec<PluginState>>,
) -> FleetDiagnostics {
    let diagnostics: Vec<NodeDiagnostics> = nodes
        .iter()
        .map(|node| {
            evaluate_node(
                node,
                nodes,
                plugin_states
                    .get(&node.id)
                    .map_or(&[] as &[PluginState], Vec::as_slice),
            )
        })
        .collect();

    let ready_nodes = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.is_ready())
        .count();
    let warning_count = diagnostics.iter().map(NodeDiagnostics::warning_count).sum();
    let critical_count = diagnostics
        .iter()
        .map(NodeDiagnostics::critical_count)
        .sum();
    let score = if diagnostics.is_empty() {
        100
    } else {
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.score)
            .sum::<usize>()
            / diagnostics.len()
    };

    FleetDiagnostics {
        score,
        ready_nodes,
        warning_count,
        critical_count,
        nodes: diagnostics,
    }
}

pub fn evaluate_node(
    node: &NodeConfig,
    all_nodes: &[NodeConfig],
    plugin_states: &[PluginState],
) -> NodeDiagnostics {
    let mut checks = Vec::new();
    checks.extend(binary_checks(node));
    checks.extend(super::checks::managed_config_checks(node, None));
    checks.extend(config_checks(node, plugin_states));
    checks.push(version_check(node));
    checks.push(status_check(node));
    checks.extend(port_checks(node, all_nodes));
    checks.extend(plugin_checks(node, plugin_states));

    let penalty = checks
        .iter()
        .map(|check| check.severity.score_penalty())
        .sum::<usize>();
    let score = 100usize.saturating_sub(penalty);

    NodeDiagnostics {
        node_id: node.id.clone(),
        node_name: node.name.clone(),
        score,
        checks,
    }
}
