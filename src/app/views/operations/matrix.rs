use eframe::egui;

use crate::{
    diagnostics::{CheckSeverity, FleetDiagnostics, PortMatrixRow},
    types::{Network, NodeStatus},
};

use super::{
    super::super::{
        paging::page_count,
        text::truncate_middle,
        theme::{muted_text, status_color},
        widgets::{empty_state, pagination_bar},
        NeoNexusApp, PORT_MATRIX_PAGE_SIZE,
    },
    helpers::severity_color,
};

impl NeoNexusApp {
    pub(super) fn render_port_matrix(&mut self, ui: &mut egui::Ui, diagnostics: &FleetDiagnostics) {
        if self.nodes.is_empty() {
            empty_state(ui, "No ports", "Create a node to inspect network bindings.");
            return;
        }

        render_port_filters(self, ui);
        self.clamp_port_matrix_page(diagnostics);
        let rows = self.filtered_port_matrix_rows(diagnostics);
        if rows.is_empty() {
            empty_state(ui, "No matching ports", "Adjust the port matrix filter.");
            return;
        }

        let total_pages = page_count(rows.len(), PORT_MATRIX_PAGE_SIZE);
        self.port_matrix_page = self.port_matrix_page.min(total_pages - 1);
        pagination_bar(ui, &mut self.port_matrix_page, total_pages, rows.len());
        ui.separator();

        let start = self.port_matrix_page * PORT_MATRIX_PAGE_SIZE;
        render_port_table(self, ui, &rows, start);
    }
}

fn render_port_filters(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Status").color(muted_text()));
        status_button(app, ui, "All", None);
        for status in NodeStatus::ALL {
            status_button(app, ui, &status.to_string(), Some(status));
        }
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Network").color(muted_text()));
        network_button(app, ui, "All", None);
        for network in Network::ALL {
            network_button(app, ui, &network.to_string(), Some(network));
        }
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Health").color(muted_text()));
        health_button(app, ui, "All", None);
        health_button(app, ui, "Blocked", Some(CheckSeverity::Critical));
        health_button(app, ui, "Clear", Some(CheckSeverity::Pass));
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.port_matrix_query).hint_text("Search"),
    );
    if response.changed() {
        app.port_matrix_page = 0;
    }
    ui.separator();
}

fn status_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    status: Option<NodeStatus>,
) {
    if ui
        .selectable_label(app.port_matrix_status_filter == status, label)
        .clicked()
    {
        app.port_matrix_status_filter = status;
        app.port_matrix_page = 0;
    }
}

fn network_button(app: &mut NeoNexusApp, ui: &mut egui::Ui, label: &str, network: Option<Network>) {
    if ui
        .selectable_label(app.port_matrix_network_filter == network, label)
        .clicked()
    {
        app.port_matrix_network_filter = network;
        app.port_matrix_page = 0;
    }
}

fn health_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    health: Option<CheckSeverity>,
) {
    if ui
        .selectable_label(app.port_matrix_health_filter == health, label)
        .clicked()
    {
        app.port_matrix_health_filter = health;
        app.port_matrix_page = 0;
    }
}

fn render_port_table(
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

fn render_port_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, row: &PortMatrixRow) {
    let selected = app.selected_node.as_deref() == Some(row.node_id.as_str());
    if ui
        .selectable_label(selected, truncate_middle(&row.node_name, 22))
        .clicked()
    {
        app.selected_node = Some(row.node_id.clone());
    }
    ui.label(row.network.to_string());
    ui.label(row.rpc_port.to_string());
    ui.label(row.p2p_port.to_string());
    ui.label(
        row.ws_port
            .map_or_else(|| "-".to_string(), |port| port.to_string()),
    );
    ui.label(
        egui::RichText::new(row.status.to_string())
            .color(status_color(row.status))
            .strong(),
    );
    ui.label(
        egui::RichText::new(row.health.label())
            .color(severity_color(row.health))
            .strong(),
    );
}
