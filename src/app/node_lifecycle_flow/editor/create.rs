use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn create_node(&mut self) {
        let input = match self.fleet.draft.to_new_node() {
            Ok(input) => input,
            Err(error) => {
                self.session.notice = Some(error.to_string());
                return;
            }
        };

        match self.repository.create_node(input) {
            Ok(node) => {
                self.fleet.selected_node = Some(node.id.clone());
                self.fleet.draft = NodeDraft::default();
                self.session.notice = Some("Node saved".to_string());
                self.record_node_event(
                    &node,
                    EventKind::NodeCreated,
                    EventSeverity::Info,
                    format!("{} created", node.name),
                );
                self.reload_nodes();
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
