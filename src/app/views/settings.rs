mod metrics;
mod monitors;
mod page;
mod policies;
mod storage;

use eframe::egui;

use super::super::NeoNexusApp;

pub(super) const PANEL_GAP: f32 = 8.0;

impl NeoNexusApp {
    pub(super) fn render_settings(&mut self, ui: &mut egui::Ui) {
        page::render_settings(self, ui);
    }
}
