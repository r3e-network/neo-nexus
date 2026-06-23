mod actions;
mod filters;
mod list;

use eframe::egui;

use crate::app::{domain::RuntimeEventFilter, theme};

use super::super::super::{widgets::empty_state, NeoNexusApp, EVENT_JOURNAL_LIMIT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EventJournalCounts {
    total_matches: usize,
    total_events: usize,
}

impl NeoNexusApp {
    pub(super) fn render_event_journal(&mut self, ui: &mut egui::Ui) {
        let filter = RuntimeEventFilter::new(
            self.event_severity_filter,
            self.event_query.clone(),
            EVENT_JOURNAL_LIMIT,
        );
        let total_matches = match self.repository.count_events(&filter) {
            Ok(count) => count,
            Err(error) => {
                ui.label(egui::RichText::new(error.to_string()).color(theme::danger()));
                return;
            }
        };
        let total_events = match self.repository.count_events(&RuntimeEventFilter::default()) {
            Ok(count) => count,
            Err(error) => {
                ui.label(egui::RichText::new(error.to_string()).color(theme::danger()));
                return;
            }
        };
        let events = match self.repository.list_events(filter) {
            Ok(events) => events,
            Err(error) => {
                ui.label(egui::RichText::new(error.to_string()).color(theme::danger()));
                return;
            }
        };

        let counts = EventJournalCounts {
            total_matches,
            total_events,
        };

        filters::render_event_filters(self, ui, counts);
        actions::render_event_actions(self, ui, counts);
        ui.separator();

        if events.is_empty() {
            empty_state(
                ui,
                "No events",
                "Adjust the filter or operate the workspace.",
            );
            return;
        }

        list::render_event_list(self, ui, &events);
    }
}
