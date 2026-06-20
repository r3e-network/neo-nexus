mod details;
mod filter;
mod import;
mod layout;
mod metrics;
mod registry;

use eframe::egui;

use super::super::{widgets::panel, NeoNexusApp};

impl NeoNexusApp {
    pub(super) fn render_wallets(&mut self, ui: &mut egui::Ui) {
        self.ensure_valid_neo_wallet_profile_selection();
        metrics::render_wallet_metrics(ui, &self.neo_wallet_profiles);

        ui.add_space(10.0);
        let layout = layout::wallet_pane_layout(ui.available_size());
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(layout.import_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Import wallet profile", |ui| {
                        import::render_wallet_profile_import_form(self, ui);
                    });
                },
            );

            ui.add_space(layout.gap);

            ui.allocate_ui_with_layout(
                egui::vec2(layout.registry_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), layout.registry_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "Profile registry", |ui| {
                                registry::render_wallet_profile_registry(self, ui);
                            });
                        },
                    );
                    ui.add_space(layout.gap);
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            panel(ui, "Selected profile", |ui| {
                                details::render_selected_wallet_profile(self, ui);
                            });
                        },
                    );
                },
            );
        });
    }
}
