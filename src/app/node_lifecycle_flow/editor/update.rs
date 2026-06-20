use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn update_selected_node(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.notice = Some("Select a node before updating it".to_string());
            return;
        };

        if node.status == NodeStatus::Running {
            self.notice = Some("Stop the selected node before editing it".to_string());
            return;
        }

        let input = match self.draft.to_new_node() {
            Ok(input) => input,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };

        match self.repository.update_node(&node.id, input) {
            Ok(updated) => {
                self.selected_node = Some(updated.id.clone());
                self.notice = Some(format!("{} updated", updated.name));
                self.record_node_event(
                    &updated,
                    EventKind::NodeUpdated,
                    EventSeverity::Info,
                    format!("{} updated", updated.name),
                );
                self.reload_nodes();
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
