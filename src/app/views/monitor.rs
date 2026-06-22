mod filter;
mod metrics;
mod pressure;
mod processes;
mod section;
mod telemetry;

use eframe::egui;

use super::super::{
    theme,
    widgets::{panel, segmented_control},
    NeoNexusApp,
};

pub(in crate::app) use section::MonitorSection;

impl NeoNexusApp {
    pub(super) fn render_monitor(&mut self, ui: &mut egui::Ui) {
        metrics::render_monitor_metrics(self, ui);

        ui.add_space(theme::MD);
        let mut index = self.monitor_section as usize;
        let labels = MonitorSection::ALL.map(MonitorSection::label);
        if segmented_control(ui, &labels, &mut index) {
            self.monitor_section = MonitorSection::ALL[index];
        }
        ui.add_space(theme::MD);

        match self.monitor_section {
            MonitorSection::Pressure => panel(ui, "System pressure", |ui| {
                pressure::render_system_pressure(self, ui);
            }),
            MonitorSection::Telemetry => panel(ui, "Telemetry health", |ui| {
                telemetry::render_telemetry_health(self, ui);
            }),
            MonitorSection::Processes => panel(ui, "Managed processes", |ui| {
                processes::render_process_metrics(self, ui);
            }),
        }
    }
}
