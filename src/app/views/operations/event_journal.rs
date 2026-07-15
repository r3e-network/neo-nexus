mod actions;
mod filters;
mod list;

use eframe::egui;

use crate::app::{
    domain::{count_workspace_events, list_workspace_events, RuntimeEventFilter},
    theme,
    widgets::empty_state,
};

use super::super::super::{NeoNexusApp, EVENT_JOURNAL_LIMIT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EventJournalCounts {
    total_matches: usize,
    total_events: usize,
}

impl NeoNexusApp {
    pub(super) fn render_event_journal(&mut self, ui: &mut egui::Ui) {
        let filter = RuntimeEventFilter::new(
            self.operations_ui.event_severity_filter,
            self.operations_ui.event_query.clone(),
            EVENT_JOURNAL_LIMIT,
        );
        let total_matches = match count_workspace_events(&self.repository, &filter) {
            Ok(count) => count,
            Err(error) => {
                ui.label(
                    egui::RichText::new(error.to_string()).color(theme::danger()),
                );
                return;
            }
        };
        let total_events =
            match count_workspace_events(&self.repository, &RuntimeEventFilter::default()) {
                Ok(count) => count,
                Err(error) => {
                    ui.label(
                        egui::RichText::new(error.to_string()).color(theme::danger()),
                    );
                    return;
                }
            };
        let events = match list_workspace_events(&self.repository, filter) {
            Ok(events) => events,
            Err(error) => {
                ui.label(
                    egui::RichText::new(error.to_string()).color(theme::danger()),
                );
                return;
            }
        };

        let counts = EventJournalCounts {
            total_matches,
            total_events,
        };

        ui.horizontal(|ui| {
            ui.label(theme::muted_body(format!(
                "Showing {total_matches} of {total_events} recorded events"
            )));
        });
        ui.add_space(theme::SM);

        filters::render_event_filters(self, ui, counts);
        actions::render_event_actions(self, ui, counts);
        ui.add_space(theme::SM);

        if events.is_empty() {
            empty_state(
                ui,
                "No events",
                "Operate nodes or clear filters to populate the journal.",
            );
            return;
        }

        list::render_event_list(self, ui, &events);
    }
}
