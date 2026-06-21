use eframe::egui;

use crate::app::{domain::EventSeverity, NeoNexusApp};

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
            egui::TextEdit::singleline(&mut app.event_query),
        );
        if response.changed() {
            app.event_page = 0;
            app.selected_event = None;
        }
    });
}

fn render_severity_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui
            .selectable_label(app.event_severity_filter.is_none(), "All")
            .clicked()
        {
            app.event_severity_filter = None;
            app.event_page = 0;
            app.selected_event = None;
        }
        for severity in EventSeverity::ALL {
            if ui
                .selectable_label(
                    app.event_severity_filter == Some(severity),
                    severity.label(),
                )
                .clicked()
            {
                app.event_severity_filter = Some(severity);
                app.event_page = 0;
                app.selected_event = None;
            }
        }
    });
}
