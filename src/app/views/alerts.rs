mod history;
mod layout;
mod metrics;
mod policy;

use eframe::egui;

use super::super::{widgets::panel, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_alerts(&mut self, ui: &mut egui::Ui) {
        let deliveries = self
            .repository
            .list_alert_deliveries(100)
            .unwrap_or_default();
        let summary = metrics::alert_delivery_summary(&deliveries);

        metrics::render_alert_metrics(
            ui,
            &self.alert_routing_policy,
            self.alert_delivery_pending,
            summary,
        );

        ui.add_space(10.0);
        let layout = layout::alert_pane_layout(ui.available_size());
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(layout.policy_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Route policy", |ui| {
                        policy::render_alert_policy_editor(self, ui);
                    });
                },
            );

            ui.add_space(layout.gap);

            ui.allocate_ui_with_layout(
                egui::vec2(layout.history_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Delivery history", |ui| {
                        history::render_alert_delivery_history(self, ui, &deliveries);
                    });
                },
            );
        });
    }
}
