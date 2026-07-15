use eframe::egui;

use crate::app::{
    domain::{RuntimePackageManager, RuntimePlatform},
    text::short_path,
    widgets::fact,
    NeoNexusApp,
};

pub(super) fn render_install_summary(app: &NeoNexusApp, ui: &mut egui::Ui) {
    fact(
        ui,
        "Downloads",
        &short_path(&app.runtime_download_dir(), 46),
    );
    fact(ui, "Root", &short_path(&app.runtime_install_root(), 46));
    fact(ui, "Candidates", &upgrade_candidates_label(app));
}

fn upgrade_candidates_label(app: &NeoNexusApp) -> String {
    let installations = app.runtime_installations();
    let platform = RuntimePlatform::current();
    app.fleet.nodes
        .iter()
        .filter(|node| {
            RuntimePackageManager::plan_node_upgrade(node, &installations, &platform).is_some()
        })
        .count()
        .to_string()
}
