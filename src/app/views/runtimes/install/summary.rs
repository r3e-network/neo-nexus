use eframe::egui;

use crate::app::{
    domain::{RuntimePackageManager, RuntimePlatform},
    text::short_path,
    theme,
    widgets::fact,
    NeoNexusApp,
};

pub(super) fn render_install_summary(app: &NeoNexusApp, ui: &mut egui::Ui) {
    ui.label(theme::label_caption("Workspace paths"));
    ui.add_space(theme::XS);
    fact(
        ui,
        "Downloads",
        &short_path(&app.runtime_download_dir(), 46),
    );
    fact(ui, "Install root", &short_path(&app.runtime_install_root(), 46));
    fact(ui, "Upgrade candidates", &upgrade_candidates_label(app));
}

fn upgrade_candidates_label(app: &NeoNexusApp) -> String {
    let installations = app.runtime_installations();
    let platform = RuntimePlatform::current();
    let count = app
        .fleet
        .nodes
        .iter()
        .filter(|node| {
            RuntimePackageManager::plan_node_upgrade(node, &installations, &platform).is_some()
        })
        .count();
    format!("{count} node(s)")
}
