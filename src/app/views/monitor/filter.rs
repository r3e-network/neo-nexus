use eframe::egui;

use crate::app::domain::ProcessStateFilter;
use crate::app::{
    theme,
    widgets::{chip_pill, filter_bar, filter_chip},
    NeoNexusApp,
};

pub(super) fn render_process_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(theme::muted_body("State"));
        chip_pill(ui, |ui| {
            state_button(app, ui, "All", None);
            state_button(app, ui, "Observed", Some(ProcessStateFilter::Observed));
            state_button(app, ui, "Missing", Some(ProcessStateFilter::Missing));
        });
    });
    ui.add_space(theme::XS);
    ui.horizontal(|ui| {
        ui.label(theme::muted_body("Pressure"));
        chip_pill(ui, |ui| {
            if filter_chip(ui, "High CPU", app.monitor_process_high_cpu_filter) {
                app.monitor_process_high_cpu_filter = !app.monitor_process_high_cpu_filter;
                app.monitor_process_page = 0;
            }
            if filter_chip(ui, "High RSS", app.monitor_process_high_memory_filter) {
                app.monitor_process_high_memory_filter = !app.monitor_process_high_memory_filter;
                app.monitor_process_page = 0;
            }
        });
    });
    ui.add_space(theme::XS);
    if filter_bar(ui, &mut app.monitor_process_query, "Search processes") {
        app.monitor_process_page = 0;
    }
    ui.add_space(theme::SM);
}

fn state_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    state: Option<ProcessStateFilter>,
) {
    if filter_chip(ui, label, app.monitor_process_state_filter == state) {
        app.monitor_process_state_filter = state;
        app.monitor_process_page = 0;
    }
}
