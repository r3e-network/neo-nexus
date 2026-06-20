use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn request_delete_selected_node(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.notice = Some("Select a node before deleting it".to_string());
            return;
        };

        if node.status == NodeStatus::Running {
            self.notice = Some("Stop the selected node before deleting it".to_string());
            return;
        }

        self.pending_delete_node = Some(node.id);
    }

    pub(in crate::app) fn confirm_delete_node(&mut self) {
        let Some(node_id) = self.pending_delete_node.clone() else {
            return;
        };

        let node_name = self
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .map_or_else(|| "node".to_string(), |node| node.name.clone());

        match self.repository.delete_node(&node_id) {
            Ok(()) => {
                self.record_event(
                    Some(node_id.clone()),
                    Some(node_name.clone()),
                    EventKind::NodeDeleted,
                    EventSeverity::Warning,
                    format!("{node_name} deleted"),
                );
                self.notice = Some(format!("{node_name} deleted"));
                self.pending_delete_node = None;
                if self.selected_node.as_deref() == Some(node_id.as_str()) {
                    self.selected_node = None;
                }
                self.reload_nodes();
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn cancel_delete_node(&mut self) {
        self.pending_delete_node = None;
    }
}
