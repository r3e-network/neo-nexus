use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn select_event(&mut self, event: &RuntimeEvent) {
        self.selected_event = Some(event.id);
        if let Some(node_id) = event.node_id.as_ref() {
            self.selected_node = Some(node_id.clone());
        }
    }

    pub(in crate::app) fn ensure_valid_event_selection(&mut self, events: &[RuntimeEvent]) {
        let selected_exists = self
            .selected_event
            .is_some_and(|id| events.iter().any(|event| event.id == id));
        if !selected_exists {
            self.selected_event = events.first().map(|event| event.id);
        }
    }

    pub(in crate::app) fn selected_event_from(
        &self,
        events: &[RuntimeEvent],
    ) -> Option<RuntimeEvent> {
        let selected_id = self.selected_event?;
        events.iter().find(|event| event.id == selected_id).cloned()
    }
}
