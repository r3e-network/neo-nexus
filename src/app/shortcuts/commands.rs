use super::{nodes::NODE_SHORTCUT_PAGE_SIZE, views, AppShortcut, NeoNexusApp, View};

impl NeoNexusApp {
    pub(in crate::app) fn apply_application_shortcut(&mut self, shortcut: AppShortcut) {
        match shortcut {
            AppShortcut::ReloadWorkspace => {
                self.reload_workspace_data();
                self.notice = Some("Workspace reloaded".to_string());
            }
            AppShortcut::NewNode => {
                self.selected_view = View::Nodes;
                self.notice = Some("Node studio ready".to_string());
            }
            AppShortcut::StartSelectedNode => self.start_selected_node(),
            AppShortcut::StopSelectedNode => self.stop_selected_node(),
            AppShortcut::RestartSelectedNode => self.restart_selected_node(),
            AppShortcut::ToggleSelectedNode => self.toggle_selected_node_lifecycle(),
            AppShortcut::PreviousNode => self.shift_selected_node(-1),
            AppShortcut::NextNode => self.shift_selected_node(1),
            AppShortcut::PreviousNodePage => {
                self.shift_selected_node(-(NODE_SHORTCUT_PAGE_SIZE as isize));
            }
            AppShortcut::NextNodePage => {
                self.shift_selected_node(NODE_SHORTCUT_PAGE_SIZE as isize);
            }
            AppShortcut::FirstNode => self.select_node_index(0),
            AppShortcut::LastNode => {
                if let Some(index) = self.visible_node_count().checked_sub(1) {
                    self.select_node_index(index);
                } else {
                    self.notice = Some("No nodes to select".to_string());
                }
            }
            AppShortcut::NextView => self.selected_view = views::next_view(self.selected_view),
            AppShortcut::PreviousView => {
                self.selected_view = views::previous_view(self.selected_view);
            }
            AppShortcut::SelectView(view) => self.selected_view = view,
        }
    }

    pub(super) fn toggle_selected_node_lifecycle(&mut self) {
        let Some(status) = self.selected_node().map(|node| node.status) else {
            self.notice = Some("Select a node before changing lifecycle state".to_string());
            return;
        };

        if status.is_running() {
            self.stop_selected_node();
        } else {
            self.start_selected_node();
        }
    }
}
