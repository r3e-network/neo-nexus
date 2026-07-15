use eframe::egui;

use super::super::super::{render_toast_strip, text::short_path, theme, NeoNexusApp};

impl NeoNexusApp {
    pub(in crate::app) fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(theme::SM);
            ui.label(theme::muted_body(format!("Nodes: {}", self.fleet.nodes.len())));
            ui.separator();
            let running = self.running_node_count();
            if running > 0 {
                ui.label(
                    theme::muted_body(format!("Running: {running}")).color(theme::success()),
                );
                ui.separator();
            }
            ui.label(theme::muted_body(format!(
                "Wallets: {}",
                self.neo_wallet_profiles.len()
            )));
            ui.separator();
            ui.label(theme::muted_body(format!(
                "CPU: {:.0}%",
                self.metrics_snapshot.system.cpu_usage_percent
            )));
            ui.separator();
            ui.label(theme::muted_body(format!(
                "Mem: {:.0}%",
                self.metrics_snapshot.system.memory_usage_percent
            )));
            ui.separator();
            ui.label(theme::muted_body(format!(
                "Database: {}",
                short_path(self.repository.db_path(), 48)
            )));
            if !self.async_bus.rpc_health_pending.is_empty() {
                ui.separator();
                ui.label(theme::muted_body(format!(
                    "RPC probes: {}",
                    self.async_bus.rpc_health_pending.len()
                )));
            }
            if !self.async_bus.remote_federation_pending.is_empty() {
                ui.separator();
                ui.label(theme::muted_body(format!(
                    "Federation probes: {}",
                    self.async_bus.remote_federation_pending.len()
                )));
            }
            if self.async_bus.alert_delivery_pending > 0 {
                ui.separator();
                ui.label(theme::muted_body(format!(
                    "Alerts: {}",
                    self.async_bus.alert_delivery_pending
                )));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(theme::SM);
                render_toast_strip(ui, &self.session.toasts);
            });
        });
    }
}
