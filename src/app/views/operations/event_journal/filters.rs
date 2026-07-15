use eframe::egui;

use crate::app::{domain::EventSeverity, widgets::chip_pill, NeoNexusApp};

use super::EventJournalCounts;

pub(super) fn render_event_filters(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    counts: EventJournalCounts,
) {
    render_search(app, ui);
    render_severity_filter(app, ui);
    ui.horizontal(|ui| {
        ui.label(format!(
            "Matches: {}/{}",
            counts.total_matches, counts.total_events
        ));
    });
}

fn render_search(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Search");
        let response = ui.add_sized(
            [ui.available_width().max(140.0), 24.0],
            egui::TextEdit::singleline(&mut app.operations_ui.event_query),
        );
        if response.changed() {
            app.operations_ui.event_page = 0;
            app.operations_ui.selected_event = None;
        }
    });
}

fn render_severity_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        chip_pill(ui, |ui| {
            if ui
                .selectable_label(app.operations_ui.event_severity_filter.is_none(), "All")
                .clicked()
            {
                app.operations_ui.event_severity_filter = None;
                app.operations_ui.event_page = 0;
                app.operations_ui.selected_event = None;
            }
            for severity in EventSeverity::ALL {
                if ui
                    .selectable_label(
                        app.operations_ui.event_severity_filter == Some(severity),
                        severity.label(),
                    )
                    .clicked()
                {
                    app.operations_ui.event_severity_filter = Some(severity);
                    app.operations_ui.event_page = 0;
                    app.operations_ui.selected_event = None;
                }
            }
        });
    });
}
