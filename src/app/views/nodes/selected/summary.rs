use eframe::egui;

use crate::{
    app::{
        domain::{NodeConfig, NodeStatus},
        text::{non_empty, short_path, truncate_middle},
        theme::status_color,
        widgets::{fact, render_node_fact_sheet},
        NeoNexusApp,
    },
    argv::format_argv,
};

pub(super) fn render_node_summary(app: &NeoNexusApp, ui: &mut egui::Ui, node: &NodeConfig) {
    render_node_header(ui, &node.name, node.status);
    ui.separator();
    render_node_fact_sheet(ui, node);
    fact(ui, "Binary", &short_path(&node.binary_path, 40));
    fact(
        ui,
        "Args",
        &non_empty(&truncate_middle(&format_argv(&node.args), 40), "-"),
    );
    fact(
        ui,
        "Managed",
        &short_path(&app.managed_config_path(node), 40),
    );
}

fn render_node_header(ui: &mut egui::Ui, name: &str, status: NodeStatus) {
    ui.horizontal(|ui| {
        ui.heading(truncate_middle(name, 24));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(status.label())
                    .color(status_color(status))
                    .strong(),
            );
        });
    });
}
