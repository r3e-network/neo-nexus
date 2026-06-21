use eframe::egui;

use super::{menu::render_application_menu, NeoNexusApp};
use crate::app::theme::muted_text;

impl NeoNexusApp {
    pub(super) fn render_application_brand_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label(egui::RichText::new("NeoNexus").strong().size(19.0));
            ui.label(egui::RichText::new("Native Rust application").color(muted_text()));

            ui.separator();
            render_application_menu(self, ui);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .color(muted_text()),
                );
                ui.separator();
                let theme_label = self.theme.toggle_label();
                if ui
                    .button(theme_label)
                    .on_hover_text("Switch the workbench colour theme")
                    .clicked()
                {
                    self.toggle_theme();
                }
                ui.separator();
                ui.label("Linux  macOS  Windows");
            });
        });
    }
}
