use eframe::egui;

use crate::{
    app::{
        text::{short_path, truncate_middle},
        widgets::fact,
    },
    runtime::{RuntimeCatalogUpgradePlan, RuntimeInstallation, RuntimeUpgradePlan},
    types::NodeConfig,
};

pub(super) fn render_node_runtime_facts(
    ui: &mut egui::Ui,
    node: &NodeConfig,
    selected_installation: Option<&RuntimeInstallation>,
) {
    fact(ui, "Node", &truncate_middle(&node.name, 34));
    fact(ui, "Status", node.status.label());
    fact(ui, "Runtime", &node.node_type.to_string());
    fact(ui, "Current", &node.runtime_version);
    if let Some(installation) = selected_installation {
        fact(ui, "Package", &truncate_middle(&installation.label, 34));
        fact(
            ui,
            "Trust",
            if installation.signature_verified {
                "signed"
            } else {
                "hash-only"
            },
        );
        fact(ui, "Target", &installation.version);
        fact(ui, "Binary", &short_path(&installation.binary_path, 48));
    } else {
        fact(ui, "Package", "none selected");
        fact(ui, "Trust", "-");
        fact(ui, "Target", "-");
        fact(ui, "Binary", "-");
    }
}

pub(super) fn render_runtime_recommendations(
    ui: &mut egui::Ui,
    installed_plan: Option<&RuntimeUpgradePlan>,
    catalog_plan: Option<&RuntimeCatalogUpgradePlan>,
) {
    ui.separator();
    if let Some(plan) = installed_plan {
        fact(ui, "Installed rec", &plan.to_version);
        fact(ui, "Target bin", &short_path(&plan.to_binary_path, 48));
    } else {
        fact(ui, "Installed rec", "current");
    }
    if let Some(plan) = catalog_plan {
        fact(ui, "Catalog rec", &plan.to_version);
        fact(ui, "Release", &truncate_middle(&plan.release.label, 34));
    } else {
        fact(ui, "Catalog rec", "none");
        fact(ui, "Release", "-");
    }
}
