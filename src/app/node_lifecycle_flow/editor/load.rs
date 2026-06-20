use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn load_selected_node_into_draft(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.notice = Some("Select a node before editing it".to_string());
            return;
        };

        self.draft.load_from_node(&node);
        self.pending_delete_node = None;
        self.selected_view = View::Nodes;
        self.notice = Some(format!("{} loaded into Node Studio", node.name));
    }
}
