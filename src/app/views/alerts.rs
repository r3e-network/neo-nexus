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
        let pending = self.async_bus.alert_delivery_pending;

        // No metric row. Routing, threshold and target restated the route the
        // editor's own "Saved" line already gives, and the two figures that were
        // not duplicated — deliveries pending, and delivered/failed — describe
        // the history list, so they moved into its header. The ~90pt this frees
        // is what let the editor fit a panel that does not scroll.
        // `horizontal` inserts `item_spacing.x` between its children, on top of
        // the gap applied explicitly below. Subtracting it here keeps the two
        // panes inside the column without zeroing the spacing, which would
        // inherit into every label and control they contain.
        let row_spacing = ui.spacing().item_spacing.x;
        let available = ui.available_size() - egui::vec2(row_spacing * 2.0, 0.0);
        let layout = layout::alert_pane_layout(available);
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(layout.policy_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    // The allocation is a request; a stretch card measures its
                    // own room and can still come out wider. Bounding the pane
                    // is what actually holds it inside the column.
                    ui.set_max_width(layout.policy_width);
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
                    ui.set_max_width(layout.history_width);
                    panel(ui, "Delivery history", |ui| {
                        history::render_alert_delivery_history(
                            self,
                            ui,
                            &deliveries,
                            summary,
                            pending,
                        );
                    });
                },
            );
        });
    }
}
