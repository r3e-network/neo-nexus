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
    /// Lifecycle control for the selected node, plus one way through to the
    /// node's own workspace.
    ///
    /// The six per-tab shortcuts that used to live here (Studio, Config, Logs,
    /// Plugins, Health, Network) were a second copy of the Nodes page's own tab
    /// strip. Two rows of duplicate navigation is what pushed this column past
    /// the bottom of a panel that does not scroll, taking the Runtime card with
    /// it — so the inspector now opens the workspace and lets the tab strip
    /// there do the job it already does.
    pub(super) fn render_inspector_actions(&mut self, ui: &mut egui::Ui, node: &NodeConfig) {
        ui.label(theme::label_caption("Lifecycle"));
        ui.add_space(theme::XS);
        let running = node.status.is_running();
        ui.horizontal_wrapped(|ui| {
            if running {
                secondary_button_enabled(ui, "Start", false)
                    .on_hover_text("Node is already running");
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
        ui.add_space(theme::SM);
        ui.horizontal_wrapped(|ui| {
            if secondary_button(ui, "Open node")
                .on_hover_text(
                    "Open this node's workspace: definition, config, logs, plugins, health",
                )
                .clicked()
            {
                self.load_selected_node_into_draft();
                self.open_node_workspace_tab(NodeWorkspaceTab::Studio);
            }
            if secondary_button(ui, "Network")
                .on_hover_text("Open the Network hub")
                .clicked()
            {
                self.session.selected_view = View::Federation;
            }
        });
    }
}
