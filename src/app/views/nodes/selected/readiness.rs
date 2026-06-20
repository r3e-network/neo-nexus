use eframe::egui;

use crate::{
    app::{text::truncate_middle, widgets::fact, NeoNexusApp},
    diagnostics::{evaluate_launch_readiness, evaluate_restart_readiness, CheckSeverity},
    types::{NodeConfig, NodeStatus},
};

pub(super) fn render_readiness(app: &NeoNexusApp, ui: &mut egui::Ui, node: &NodeConfig) {
    let plugins = app
        .repository
        .list_plugin_states(&node.id)
        .unwrap_or_default();
    let readiness = if node.status == NodeStatus::Running {
        evaluate_restart_readiness(
            node,
            &app.nodes,
            &plugins,
            app.managed_config_path(node),
            app.node_work_dir(node),
        )
    } else {
        evaluate_launch_readiness(
            node,
            &app.nodes,
            &plugins,
            app.managed_config_path(node),
            app.node_work_dir(node),
        )
    };
    let readiness_label = if node.status == NodeStatus::Running {
        "Restart Ready"
    } else {
        "Launch Ready"
    };
    fact(ui, readiness_label, readiness.status_label());
    if let Some(check) = readiness.checks.iter().find(|check| {
        matches!(
            check.severity,
            CheckSeverity::Critical | CheckSeverity::Warning
        )
    }) {
        fact(ui, "Finding", &truncate_middle(&check.detail, 40));
    }
}
