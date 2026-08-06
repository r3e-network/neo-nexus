use eframe::egui;

use crate::app::{
    domain::{RuntimePackageManager, RuntimePlatform},
    text::short_path,
    theme,
    widgets::metric_grid,
    NeoNexusApp,
};

/// Where runtime packages are downloaded to and installed into, plus how many
/// nodes could move to a newer runtime. Rendered as one row rather than three
/// stacked facts: this is reference detail under a list, not the subject of the
/// page, and three full-width rows is more height than it earns.
pub(in crate::app) fn render_install_summary(app: &NeoNexusApp, ui: &mut egui::Ui) {
    ui.label(theme::label_caption("Workspace paths"));
    ui.add_space(theme::XS);
    metric_grid(
        ui,
        &[
            ("Downloads", short_path(&app.runtime_download_dir(), 30)),
            ("Install root", short_path(&app.runtime_install_root(), 30)),
            ("Upgrade candidates", upgrade_candidates_label(app)),
        ],
    );
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
