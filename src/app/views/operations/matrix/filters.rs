use eframe::egui;

use crate::app::domain::{CheckSeverity, FleetDiagnostics, Network, NodeStatus};
use crate::app::widgets::chip_pill;

use super::super::super::super::{theme::muted_text, NeoNexusApp};

pub(super) fn render_port_filters(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    diagnostics: &FleetDiagnostics,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Status").color(muted_text()));
        chip_pill(ui, |ui| {
            status_button(app, ui, "All", None);
            for status in NodeStatus::ALL {
                status_button(app, ui, status.label(), Some(status));
            }
        });
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Network").color(muted_text()));
        chip_pill(ui, |ui| {
            network_button(app, ui, "All", None);
            for network in Network::ALL {
                network_button(app, ui, &network.to_string(), Some(network));
            }
        });
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Health").color(muted_text()));
        chip_pill(ui, |ui| {
            health_button(app, ui, "All", None);
            health_button(app, ui, "Blocked", Some(CheckSeverity::Critical));
            health_button(app, ui, "Clear", Some(CheckSeverity::Pass));
        });
        ui.separator();
        if ui
            .add_enabled(
                diagnostics.critical_count > 0,
                egui::Button::new("Focus Blocked"),
            )
            .on_hover_text("Show blocked port rows and select the first conflict")
            .clicked()
        {
            app.focus_blocked_ports(diagnostics);
        }
        if ui
            .add_enabled(
                app.has_active_port_matrix_filter(),
                egui::Button::new("Clear Filters"),
            )
            .on_hover_text("Show all port rows")
            .clicked()
        {
            app.clear_port_matrix_filters(diagnostics);
        }
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
