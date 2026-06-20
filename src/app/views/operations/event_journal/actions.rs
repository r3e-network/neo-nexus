use eframe::egui;

use crate::app::{NeoNexusApp, EVENT_RETAIN_AFTER_PRUNE};

use super::EventJournalCounts;

pub(super) fn render_event_actions(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    counts: EventJournalCounts,
) {
    ui.horizontal(|ui| {
        if ui
            .add_enabled(has_active_filter(app), egui::Button::new("Clear"))
            .on_hover_text("Clear event journal filters")
            .clicked()
        {
            clear_event_filters(app);
        }
        if ui
            .add_enabled(counts.total_matches > 0, egui::Button::new("Export"))
            .on_hover_text("Export filtered audit evidence as text and JSON")
            .clicked()
        {
            app.export_event_journal_report();
        }
        if ui
            .add_enabled(can_prune_events(counts), egui::Button::new("Prune"))
            .on_hover_text("Retain only the newest bounded audit events")
            .clicked()
        {
            app.prune_event_journal();
        }
    });
}

fn has_active_filter(app: &NeoNexusApp) -> bool {
    !app.event_query.trim().is_empty() || app.event_severity_filter.is_some()
}

fn clear_event_filters(app: &mut NeoNexusApp) {
    app.event_query.clear();
    app.event_severity_filter = None;
    app.event_page = 0;
    app.selected_event = None;
}

fn can_prune_events(counts: EventJournalCounts) -> bool {
    counts.total_events > EVENT_RETAIN_AFTER_PRUNE
}
