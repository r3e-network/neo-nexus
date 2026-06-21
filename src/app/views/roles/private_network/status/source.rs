use eframe::egui;

use crate::app::{
    domain::PrivateNetworkConflict,
    text::{short_path, truncate_middle},
    widgets::fact,
    NeoNexusApp,
};

use super::super::SourceNode;

pub(in crate::app::views::roles::private_network) fn render_source_status(
    _app: &NeoNexusApp,
    ui: &mut egui::Ui,
    source: SourceNode<'_>,
    first_conflict: Option<&PrivateNetworkConflict>,
) {
    fact(
        ui,
        "Source",
        &source.map_or_else(
            || "missing runtime template".to_string(),
            |node| {
                format!(
                    "{}  {}",
                    truncate_middle(&node.name, 24),
                    short_path(&node.binary_path, 30)
                )
            },
        ),
    );
    fact(
        ui,
        "Conflicts",
        &first_conflict.map_or_else(
            || "none".to_string(),
            |conflict| truncate_middle(&conflict.detail, 58),
        ),
    );
}
