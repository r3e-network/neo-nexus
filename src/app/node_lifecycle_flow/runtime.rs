use super::*;

mod restart;
mod start;
mod stop;

impl NeoNexusApp {
    pub(in crate::app) fn start_selected_node(&mut self) {
        let Some(index) = self.selected_node_index() else {
            self.notice = Some("Select a node before starting it".to_string());
            return;
        };
        self.start_node(index);
    }

    pub(in crate::app) fn stop_selected_node(&mut self) {
        let Some(index) = self.selected_node_index() else {
            self.notice = Some("Select a node before stopping it".to_string());
            return;
        };
        self.stop_node(index);
    }

    pub(in crate::app) fn restart_selected_node(&mut self) {
        let Some(index) = self.selected_node_index() else {
            self.notice = Some("Select a running node before restarting it".to_string());
            return;
        };
        self.restart_node(index);
    }

    pub(in crate::app) fn start_node(&mut self, index: usize) {
        self.start_node_with_mode(index, StartMode::Manual);
    }
}
