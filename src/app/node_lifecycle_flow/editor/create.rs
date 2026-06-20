use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn create_node(&mut self) {
        let input = match self.draft.to_new_node() {
            Ok(input) => input,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };

        match self.repository.create_node(input) {
            Ok(node) => {
                self.selected_node = Some(node.id.clone());
                self.draft = NodeDraft::default();
                self.notice = Some("Node saved".to_string());
                self.record_node_event(
                    &node,
                    EventKind::NodeCreated,
                    EventSeverity::Info,
                    format!("{} created", node.name),
                );
                self.reload_nodes();
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
