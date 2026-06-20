use eframe::egui;

use crate::{
    catalog::PluginDefinition,
    metrics::format_bytes,
    types::{NodeConfig, NodeStatus},
};

use super::super::super::{
    text::truncate_middle,
    theme::muted_text,
    widgets::{fact, labeled_text},
    NeoNexusApp,
};

pub(super) fn render_plugin_package_installer(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    node: &NodeConfig,
    plugin: &PluginDefinition,
) {
    let installations = app
        .repository
        .list_plugin_installations(&node.id)
        .unwrap_or_default();
    let installed = installations
        .iter()
        .find(|installation| installation.plugin_id == plugin.id);
    let status = installed.map_or_else(
        || "not installed".to_string(),
        |installation| {
            format!(
                "{} files, {}",
                installation.installed_files,
                format_bytes(installation.expanded_bytes)
            )
        },
    );

    ui.label(egui::RichText::new("Package").color(muted_text()));
    fact(ui, "Installed", &status);
    if let Some(installation) = installed {
        fact(
            ui,
            "Path",
            &truncate_middle(&installation.installed_path.display().to_string(), 42),
        );
        fact(ui, "SHA-256", &truncate_middle(&installation.sha256, 42));
    }

    ui.add_space(8.0);
    labeled_text(ui, "ZIP", &mut app.plugin_package_source);
    labeled_text(ui, "SHA-256", &mut app.plugin_package_expected_sha256);

    let can_hash = !app.plugin_package_source.trim().is_empty();
    let can_install = can_hash
        && !app.plugin_package_expected_sha256.trim().is_empty()
        && !matches!(node.status, NodeStatus::Running | NodeStatus::Starting);

    ui.horizontal(|ui| {
        if ui
            .add_enabled(can_hash, egui::Button::new("Use SHA-256"))
            .clicked()
        {
            app.fill_selected_plugin_package_sha256();
        }
        if ui
            .add_enabled(can_install, egui::Button::new("Install ZIP"))
            .clicked()
        {
            app.install_selected_plugin_package();
        }
    });

    if matches!(node.status, NodeStatus::Running | NodeStatus::Starting) {
        ui.label(
            egui::RichText::new("Stop this node before replacing plugin files.")
                .color(muted_text()),
        );
    }
}
