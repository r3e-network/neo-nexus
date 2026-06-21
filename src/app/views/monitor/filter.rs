use eframe::egui;

use crate::app::domain::ProcessStateFilter;

use super::super::super::{theme::muted_text, NeoNexusApp};

pub(super) fn render_process_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("State").color(muted_text()));
        state_button(app, ui, "All", None);
        state_button(app, ui, "Observed", Some(ProcessStateFilter::Observed));
        state_button(app, ui, "Missing", Some(ProcessStateFilter::Missing));
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Pressure").color(muted_text()));
        pressure_toggle(
            ui,
            &mut app.monitor_process_high_cpu_filter,
            "High CPU",
            &mut app.monitor_process_page,
        );
        pressure_toggle(
            ui,
            &mut app.monitor_process_high_memory_filter,
            "High RSS",
            &mut app.monitor_process_page,
        );
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.monitor_process_query).hint_text("Search"),
    );
    if response.changed() {
        app.monitor_process_page = 0;
    }
    ui.separator();
}

fn state_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    state: Option<ProcessStateFilter>,
) {
    if ui
        .selectable_label(app.monitor_process_state_filter == state, label)
        .clicked()
    {
        app.monitor_process_state_filter = state;
        app.monitor_process_page = 0;
    }
}

fn pressure_toggle(ui: &mut egui::Ui, value: &mut bool, label: &str, page: &mut usize) {
    let changed = ui.checkbox(value, label).changed();
    if changed {
        *page = 0;
    }
}
