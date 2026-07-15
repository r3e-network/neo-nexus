mod definition;
mod layout;
mod selected;
mod workspace;

use eframe::egui;

use super::super::{
    widgets::{page_chrome, panel},
    NeoNexusApp,
};

pub(in crate::app) use workspace::NodeWorkspaceTab;

impl NeoNexusApp {
    pub(super) fn render_nodes(&mut self, ui: &mut egui::Ui) {
        let mut index = self.session.node_workspace_tab as usize;
        let labels = NodeWorkspaceTab::ALL.map(NodeWorkspaceTab::label);
        // Shell owns title; segments-only chrome per v3.1 contract.
        if page_chrome(ui, None, Some((&labels, &mut index))) {
            self.session.node_workspace_tab = NodeWorkspaceTab::ALL[index];
        }

        match self.session.node_workspace_tab {
            NodeWorkspaceTab::Studio => self.render_node_studio(ui),
            NodeWorkspaceTab::Config => self.render_config(ui),
            NodeWorkspaceTab::Logs => self.render_logs(ui),
            NodeWorkspaceTab::Plugins => self.render_plugins(ui),
            NodeWorkspaceTab::Health => self.render_monitor(ui),
        }
    }

    fn render_node_studio(&mut self, ui: &mut egui::Ui) {
        let layout = layout::node_pane_layout(ui.available_size());

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(layout.definition_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Node definition", |ui| {
                        definition::render_create_form(self, ui);
                    });
                },
            );

            ui.add_space(layout.gap);

            ui.allocate_ui_with_layout(
                egui::vec2(layout.selected_width, layout.height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Selected node", |ui| {
                        selected::render_selected_node_editor(self, ui);
                    });
                },
            );
        });
    }

    /// Open Nodes on a specific workspace tab (used by Home CTAs and legacy deep links).
    pub(in crate::app) fn open_node_workspace_tab(&mut self, tab: NodeWorkspaceTab) {
        self.session.node_workspace_tab = tab;
        self.session.selected_view = crate::app::view::View::Nodes;
    }
}
