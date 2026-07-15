use eframe::egui;

use crate::app::domain::NodeConfig;

use super::super::super::super::{
    theme,
    view::View,
    views::NodeWorkspaceTab,
    widgets::{primary_button, secondary_button, secondary_button_enabled},
    NeoNexusApp,
};

impl NeoNexusApp {
    pub(super) fn render_inspector_actions(&mut self, ui: &mut egui::Ui, node: &NodeConfig) {
        ui.label(theme::label_caption("Lifecycle"));
        ui.add_space(theme::XS);
        let running = node.status.is_running();
        ui.horizontal(|ui| {
            if running {
                if secondary_button_enabled(ui, "Start", false)
                    .on_hover_text("Node is already running")
                    .clicked()
                {}
            } else if primary_button(ui, "Start")
                .on_hover_text("Start this stopped node")
                .clicked()
            {
                self.start_selected_node();
            }
            if secondary_button_enabled(ui, "Stop", running)
                .on_hover_text("Stop this running node")
                .clicked()
            {
                self.stop_selected_node();
            }
            if secondary_button_enabled(ui, "Restart", running)
                .on_hover_text("Restart this running node")
                .clicked()
            {
                self.restart_selected_node();
            }
        });
        ui.add_space(theme::MD);
        ui.label(theme::label_caption("Open"));
        ui.add_space(theme::XS);
        ui.horizontal_wrapped(|ui| {
            if secondary_button(ui, "Studio")
                .on_hover_text("Edit this node definition")
                .clicked()
            {
                self.load_selected_node_into_draft();
                self.open_node_workspace_tab(NodeWorkspaceTab::Studio);
            }
            if secondary_button(ui, "Config")
                .on_hover_text("Inspect generated configuration")
                .clicked()
            {
                self.open_node_workspace_tab(NodeWorkspaceTab::Config);
            }
            if secondary_button(ui, "Logs")
                .on_hover_text("Open runtime logs")
                .clicked()
            {
                self.open_node_workspace_tab(NodeWorkspaceTab::Logs);
            }
            if secondary_button(ui, "Plugins")
                .on_hover_text("Manage node plugins")
                .clicked()
            {
                self.open_node_workspace_tab(NodeWorkspaceTab::Plugins);
            }
            if secondary_button(ui, "Health")
                .on_hover_text("Resource and process health")
                .clicked()
            {
                self.open_node_workspace_tab(NodeWorkspaceTab::Health);
            }
            if secondary_button(ui, "Network")
                .on_hover_text("Open Network hub")
                .clicked()
            {
                self.session.selected_view = View::Federation;
            }
        });
    }
}
