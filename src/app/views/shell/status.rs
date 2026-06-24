use eframe::egui;

use super::super::super::{text::short_path, theme, NeoNexusApp};

impl NeoNexusApp {
    pub(in crate::app) fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(theme::SM);
            ui.label(theme::muted_body(format!("Nodes: {}", self.nodes.len())));
            ui.separator();
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
                short_path(self.repository.db_path(), 68)
            )));
            if !self.rpc_health_pending.is_empty() {
                ui.separator();
                ui.label(theme::muted_body(format!(
                    "RPC probes: {}",
                    self.rpc_health_pending.len()
                )));
            }
            if !self.remote_federation_pending.is_empty() {
                ui.separator();
                ui.label(theme::muted_body(format!(
                    "Federation probes: {}",
                    self.remote_federation_pending.len()
                )));
            }
            if self.alert_delivery_pending > 0 {
                ui.separator();
                ui.label(theme::muted_body(format!(
                    "Alerts: {}",
                    self.alert_delivery_pending
                )));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(theme::SM);
                ui.label(theme::muted_body(self.notice.as_deref().unwrap_or("Ready")));
            });
        });
    }
}
