use eframe::egui;

use crate::app::{
    domain::{evaluate_launch_readiness, evaluate_restart_readiness, CheckSeverity, NodeConfig},
    text::truncate_middle,
    widgets::fact,
    NeoNexusApp,
};

pub(super) fn render_readiness(app: &NeoNexusApp, ui: &mut egui::Ui, node: &NodeConfig) {
    let plugins = app
        .repository
        .list_plugin_states(&node.id)
        .unwrap_or_default();
    let readiness = if node.status.is_running() {
        evaluate_restart_readiness(
            node,
            &app.fleet.nodes,
            &plugins,
            app.managed_config_path(node),
            app.node_work_dir(node),
        )
    } else {
        evaluate_launch_readiness(
            node,
            &app.fleet.nodes,
            &plugins,
            app.managed_config_path(node),
            app.node_work_dir(node),
        )
    };
    let readiness_label = if node.status.is_running() {
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
