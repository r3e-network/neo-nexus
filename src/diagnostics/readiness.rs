use std::path::Path;

use crate::{
    catalog::PluginState, launch::LaunchPlanner, port_planner::is_localhost_tcp_port_available,
    types::NodeConfig,
};

use super::{
    checks::{
        binary_checks, config_checks, launch_lifecycle_checks, managed_config_checks,
        restart_lifecycle_checks,
    },
    ports::launch_port_checks,
    CheckSeverity, DiagnosticCheck, DiagnosticResolution, LaunchReadinessReport,
};

pub fn evaluate_launch_readiness(
    node: &NodeConfig,
    all_nodes: &[NodeConfig],
    plugin_states: &[PluginState],
    managed_config_path: impl AsRef<Path>,
    working_dir: impl AsRef<Path>,
) -> LaunchReadinessReport {
    evaluate_launch_readiness_with_port_probe(
        node,
        all_nodes,
        plugin_states,
        managed_config_path,
        working_dir,
        is_localhost_tcp_port_available,
    )
}

pub fn evaluate_restart_readiness(
    node: &NodeConfig,
    all_nodes: &[NodeConfig],
    plugin_states: &[PluginState],
    managed_config_path: impl AsRef<Path>,
    working_dir: impl AsRef<Path>,
) -> LaunchReadinessReport {
    evaluate_restart_readiness_with_port_probe(
        node,
        all_nodes,
        plugin_states,
        managed_config_path,
        working_dir,
        is_localhost_tcp_port_available,
    )
}

pub fn evaluate_restart_readiness_with_port_probe<F>(
    node: &NodeConfig,
    all_nodes: &[NodeConfig],
    plugin_states: &[PluginState],
    managed_config_path: impl AsRef<Path>,
    working_dir: impl AsRef<Path>,
    is_port_available: F,
) -> LaunchReadinessReport
where
    F: FnMut(u16) -> bool,
{
    let plan = LaunchPlanner::plan(node, managed_config_path, working_dir);
    let mut checks = Vec::new();
    checks.extend(binary_checks(node));
    checks.extend(managed_config_checks(node, Some(&plan)));
    checks.extend(config_checks(node, plugin_states));
    checks.extend(restart_lifecycle_checks(node));
    checks.extend(launch_port_checks(node, all_nodes, is_port_available));
    checks.push(DiagnosticCheck::new(
        CheckSeverity::Pass,
        "Restart command",
        plan.display_command.clone(),
        DiagnosticResolution::Operations,
    ));

    LaunchReadinessReport {
        node_id: node.id.clone(),
        node_name: node.name.clone(),
        display_command: plan.display_command,
        checks,
    }
}

pub fn evaluate_launch_readiness_with_port_probe<F>(
    node: &NodeConfig,
    all_nodes: &[NodeConfig],
    plugin_states: &[PluginState],
    managed_config_path: impl AsRef<Path>,
    working_dir: impl AsRef<Path>,
    is_port_available: F,
) -> LaunchReadinessReport
where
    F: FnMut(u16) -> bool,
{
    let plan = LaunchPlanner::plan(node, managed_config_path, working_dir);
    let mut checks = Vec::new();
    checks.extend(binary_checks(node));
    checks.extend(managed_config_checks(node, Some(&plan)));
    checks.extend(config_checks(node, plugin_states));
    checks.extend(launch_lifecycle_checks(node));
    checks.extend(launch_port_checks(node, all_nodes, is_port_available));
    checks.push(DiagnosticCheck::new(
        CheckSeverity::Pass,
        "Launch command",
        plan.display_command.clone(),
        DiagnosticResolution::Operations,
    ));

    LaunchReadinessReport {
        node_id: node.id.clone(),
        node_name: node.name.clone(),
        display_command: plan.display_command,
        checks,
    }
}
