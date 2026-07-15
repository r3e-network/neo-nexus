use eframe::egui;

use super::super::super::{render_toast_strip, text::short_path, theme, NeoNexusApp};

impl NeoNexusApp {
    pub(in crate::app) fn render_status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(theme::SM);
            status_chip(ui, "Nodes", &self.fleet.nodes.len().to_string(), None);
            ui.separator();
            let running = self.running_node_count();
            status_chip(
                ui,
                "Running",
                &running.to_string(),
                if running > 0 {
                    Some(theme::success())
                } else {
                    None
                },
            );
            ui.separator();
            status_chip(
                ui,
                "Wallets",
                &self.neo_wallet_profiles.len().to_string(),
                None,
            );
            ui.separator();
            status_chip(
                ui,
                "CPU",
                &format!("{:.0}%", self.metrics_snapshot.system.cpu_usage_percent),
                pressure_tint(self.metrics_snapshot.system.cpu_usage_percent),
            );
            ui.separator();
            status_chip(
                ui,
                "Mem",
                &format!("{:.0}%", self.metrics_snapshot.system.memory_usage_percent),
                pressure_tint(self.metrics_snapshot.system.memory_usage_percent),
            );
            ui.separator();
            ui.label(theme::muted_body(format!(
                "DB {}",
                short_path(self.repository.db_path(), 42)
            )));
            if !self.async_bus.rpc_health_pending.is_empty() {
                ui.separator();
                status_chip(
                    ui,
                    "RPC",
                    &self.async_bus.rpc_health_pending.len().to_string(),
                    Some(theme::info()),
                );
            }
            if !self.async_bus.remote_federation_pending.is_empty() {
                ui.separator();
                status_chip(
                    ui,
                    "Fed",
                    &self.async_bus.remote_federation_pending.len().to_string(),
                    Some(theme::info()),
                );
            }
            if self.async_bus.alert_delivery_pending > 0 {
                ui.separator();
                status_chip(
                    ui,
                    "Alerts",
                    &self.async_bus.alert_delivery_pending.to_string(),
                    Some(theme::warning()),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(theme::SM);
                render_toast_strip(ui, &self.session.toasts);
            });
        });
    }
}

fn status_chip(ui: &mut egui::Ui, label: &str, value: &str, color: Option<egui::Color32>) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = theme::XS;
        ui.label(theme::muted_body(label));
        let text = theme::body(value).strong();
        ui.label(match color {
            Some(c) => text.color(c),
            None => text,
        });
    });
}

fn pressure_tint(percent: f32) -> Option<egui::Color32> {
    if percent >= 90.0 {
        Some(theme::danger())
    } else if percent >= 75.0 {
        Some(theme::warning())
    } else {
        None
    }
}
