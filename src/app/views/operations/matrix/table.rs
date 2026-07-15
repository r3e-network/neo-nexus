use eframe::egui;

use crate::app::domain::PortMatrixRow;

use super::super::{
    super::super::{
        text::truncate_middle,
        theme,
        widgets::{grid_header, severity_badge, status_badge},
        NeoNexusApp, PORT_MATRIX_PAGE_SIZE,
    },
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
            grid_header(
                ui,
                &["Node", "Chain", "RPC", "P2P", "WS", "Status", "Health"],
            );

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

    ui.add_space(theme::SM);
    egui::Frame::new()
        .fill(theme::card_surface())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(theme::label_caption("Selected port row"));
                ui.add_space(theme::SM);
                status_badge(ui, row.status);
                ui.add_space(theme::XS);
                severity_badge(ui, row.health);
            });
            ui.add_space(theme::SM);
            ui.label(theme::body(truncate_middle(&row.node_name, 36)).strong());
            ui.label(theme::muted_body(format!(
                "{} · RPC {} · P2P {} · WS {}",
                row.network,
                row.rpc_port,
                row.p2p_port,
                optional_port(row.ws_port)
            )));
        });
}

fn render_port_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, row: &PortMatrixRow) {
    let selected = app.fleet.selected_node.as_deref() == Some(row.node_id.as_str());
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
    status_badge(ui, row.status);
    severity_badge(ui, row.health);
}

fn optional_port(port: Option<u16>) -> String {
    port.map_or_else(|| "—".to_string(), |value| value.to_string())
}
