mod chain_duties;
mod filter;
mod metrics;
mod pressure;
mod processes;
mod section;
mod telemetry;

use eframe::egui;

use super::super::{
    theme,
    widgets::{page_chrome, panel},
    NeoNexusApp,
};

pub(in crate::app) use section::MonitorSection;

impl NeoNexusApp {
    pub(super) fn render_monitor(&mut self, ui: &mut egui::Ui) {
        // No page-level metric row. CPU and Memory are in the status bar at all
        // times, and Node CPU / Node RSS are what the Pressure section is for —
        // so above the tabs they were a duplicate on one tab and a distraction
        // on the other three, costing ~90pt on a page where two of them did not
        // fit.
        let mut index = self.sections.monitor as usize;
        let labels = MonitorSection::ALL.map(MonitorSection::label);
        if page_chrome(ui, None, Some((&labels, &mut index))) {
            self.sections.monitor = MonitorSection::ALL[index];
        }

        match self.sections.monitor {
            MonitorSection::Pressure => panel(ui, "System pressure", |ui| {
                metrics::render_pressure_metrics(self, ui);
                ui.add_space(theme::SM);
                pressure::render_system_pressure(self, ui);
            }),
            MonitorSection::Telemetry => panel(ui, "Telemetry health", |ui| {
                telemetry::render_telemetry_health(self, ui);
            }),
            MonitorSection::Processes => panel(ui, "Managed processes", |ui| {
                processes::render_process_metrics(self, ui);
            }),
            MonitorSection::ChainDuties => panel(ui, "Chain duties", |ui| {
                self.render_chain_duties(ui);
            }),
        }
    }
}
