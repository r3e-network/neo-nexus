use eframe::egui;

use crate::{
    app::NeoNexusApp,
    runtime::RuntimeCatalogUpgradePlan,
    types::{NodeConfig, NodeStatus},
};

use super::status::{catalog_upgrade_color, catalog_upgrade_state};

pub(super) fn render_runtime_actions(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    node: &NodeConfig,
    has_selected_installation: bool,
    selected_compatible: bool,
    catalog_plan: Option<&RuntimeCatalogUpgradePlan>,
) {
    let stopped = node.status == NodeStatus::Stopped;
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                has_selected_installation && selected_compatible && stopped,
                egui::Button::new("Apply to Node"),
            )
            .clicked()
        {
            app.apply_selected_runtime_to_node();
        }
        if ui
            .add_enabled(
                catalog_plan.is_some() && stopped,
                egui::Button::new("Upgrade from Catalog"),
            )
            .clicked()
        {
            app.upgrade_selected_node_from_catalog();
        }
    });
    ui.label(
        egui::RichText::new(catalog_upgrade_state(
            node.status,
            app.runtime_catalog.is_some(),
            catalog_plan.is_some(),
        ))
        .color(catalog_upgrade_color(
            node.status,
            app.runtime_catalog.is_some(),
            catalog_plan.is_some(),
        )),
    );
}
