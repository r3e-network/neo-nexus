mod actions;
mod brand;
mod menu;
mod navigation;
mod state;

use eframe::egui;

use super::super::super::{view::View, NeoNexusApp};

impl NeoNexusApp {
    pub(in crate::app) fn render_application_header(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        self.render_application_brand_row(ui);
        ui.separator();
        self.render_application_navigation_row(ui);
    }
}
