use eframe::egui;

use crate::diagnostics::PortMatrixRow;

use super::super::{
    super::super::{
        text::truncate_middle,
        theme::{muted_text, status_color},
        NeoNexusApp, PORT_MATRIX_PAGE_SIZE,
    },
    helpers::severity_color,
};

pub(super) fn render_port_table(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    rows: &[PortMatrixRow],
    start: usize,
) {
    egui::Grid::new("operations_port_matrix")
        .striped(true)
        .min_col_width(70.0)
        .show(ui, |ui| {
            ui.strong("Node");
            ui.strong("Chain");
            ui.strong("RPC");
            ui.strong("P2P");
            ui.strong("WS");
            ui.strong("Status");
            ui.strong("Health");
            ui.end_row();

            for row in rows.iter().skip(start).take(PORT_MATRIX_PAGE_SIZE) {
                render_port_row(app, ui, row);
                ui.end_row();
            }
        });
}

pub(super) fn render_selected_port_summary(
    app: &NeoNexusApp,
    ui: &mut egui::Ui,
    rows: &[PortMatrixRow],
) {
    let Some(row) = app.selected_visible_port_matrix_row(rows) else {
        return;
    };

    ui.separator();
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Selected").color(muted_text()));
        ui.label(truncate_middle(&row.node_name, 22))
            .on_hover_text(row.node_name.as_str());
        ui.label(row.network.to_string());
        ui.label(format!("RPC {}", row.rpc_port));
        ui.label(format!("P2P {}", row.p2p_port));
        ui.label(format!("WS {}", optional_port(row.ws_port)));
        ui.label(
            egui::RichText::new(row.health.label())
                .strong()
                .color(severity_color(row.health)),
        );
    });
}

fn render_port_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, row: &PortMatrixRow) {
    let selected = app.selected_node.as_deref() == Some(row.node_id.as_str());
    if ui
        .selectable_label(selected, truncate_middle(&row.node_name, 22))
        .on_hover_text(row.node_name.as_str())
        .clicked()
    {
        app.select_port_matrix_row(row);
    }
    ui.label(row.network.to_string());
    ui.label(row.rpc_port.to_string());
    ui.label(row.p2p_port.to_string());
    ui.label(optional_port(row.ws_port));
    ui.label(
        egui::RichText::new(row.status.label())
            .color(status_color(row.status))
            .strong(),
    );
    ui.label(
        egui::RichText::new(row.health.label())
            .color(severity_color(row.health))
            .strong(),
    );
}

fn optional_port(port: Option<u16>) -> String {
    port.map_or_else(|| "-".to_string(), |port| port.to_string())
}
