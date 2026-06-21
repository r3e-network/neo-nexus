use eframe::egui;

use crate::app::{
    domain::NodeConfig,
    text::{short_path, truncate_middle},
    widgets::fact,
    NeoNexusApp,
};

pub(super) fn render_target_node(app: &NeoNexusApp, ui: &mut egui::Ui) {
    let selected_node = app.selected_node();
    fact(ui, "Target node", &target_node_label(selected_node));
    if let Some(node) = selected_node {
        fact(ui, "Target data", &short_path(&app.node_data_dir(node), 70));
    } else {
        fact(ui, "Target data", "-");
    }
}

fn target_node_label(node: Option<&NodeConfig>) -> String {
    node.map_or_else(
        || "-".to_string(),
        |node| {
            format!(
                "{}  {}  {}",
                truncate_middle(&node.name, 24),
                node.node_type,
                node.network
            )
        },
    )
}
