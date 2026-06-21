use eframe::egui;

use crate::app::{
    domain::{RuntimeInstallation, RuntimePackageManager, RuntimePlatform},
    widgets::empty_state,
    NeoNexusApp,
};

use super::{
    actions::render_runtime_actions,
    facts::{render_node_runtime_facts, render_runtime_recommendations},
};

pub(super) fn render_runtime_application(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    installations: &[RuntimeInstallation],
) {
    let selected_installation = app.selected_runtime_installation(installations);
    let selected_node = app.selected_node().cloned();

    match selected_node {
        Some(node) => {
            let selected_compatible = selected_installation
                .as_ref()
                .is_some_and(|installation| node.node_type == installation.node_type);
            let installed_plan = RuntimePackageManager::plan_node_upgrade(
                &node,
                installations,
                &RuntimePlatform::current(),
            );
            let catalog_plan = app.catalog_upgrade_plan_for_node(&node);

            render_node_runtime_facts(ui, &node, selected_installation.as_ref());
            render_runtime_recommendations(ui, installed_plan.as_ref(), catalog_plan.as_ref());
            render_runtime_actions(
                app,
                ui,
                &node,
                selected_installation.is_some(),
                selected_compatible,
                catalog_plan.as_ref(),
            );
        }
        None => empty_state(ui, "No node", "Select a node from Inventory."),
    }
}
