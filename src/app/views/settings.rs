mod metrics;
mod monitors;
mod page;
mod policies;
mod section;
mod storage;

use eframe::egui;

use super::super::NeoNexusApp;

pub(in crate::app) use section::SettingsSection;

impl NeoNexusApp {
    pub(super) fn render_settings(&mut self, ui: &mut egui::Ui) {
        page::render_settings(self, ui);
    }
}
