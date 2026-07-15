mod alerts;
mod config;
mod federation;
mod logs;
mod monitor;
mod network_hub;
mod nodes;
mod operations;
mod overview;
mod plugins;
mod roles;
mod runtimes;
mod settings;
mod shell;
mod snapshots;
mod wallets;

use eframe::egui;

use super::{view::View, NeoNexusApp};

pub(in crate::app) use federation::FederationSection;
pub(in crate::app) use monitor::MonitorSection;
pub(in crate::app) use network_hub::NetworkHubSection;
pub(in crate::app) use nodes::NodeWorkspaceTab;
pub(in crate::app) use operations::OperationsSection;
pub(in crate::app) use roles::RolesSection;
pub(in crate::app) use runtimes::RuntimesSection;
pub(in crate::app) use settings::SettingsSection;
pub(in crate::app) use snapshots::SnapshotsSection;

impl NeoNexusApp {
    pub(super) fn render_workspace(&mut self, ui: &mut egui::Ui) {
        ui.add_space(2.0);
        self.normalize_navigation_for_v3();

        match self.selected_view {
            View::Summary => self.render_overview(ui),
            View::Operations => self.render_operations(ui),
            // Legacy top-level tools: normalize_navigation maps them into
            // Nodes / Runtimes / Settings tabs before this match runs.
            View::Monitor | View::Config | View::Logs | View::Plugins => self.render_nodes(ui),
            View::Alerts => self.render_settings(ui),
            View::Federation | View::Roles | View::Wallets => self.render_network_hub(ui),
            View::Settings => self.render_settings(ui),
            View::Runtimes | View::Snapshots => self.render_runtimes(ui),
            View::Nodes => self.render_nodes(ui),
        }
    }

    /// Collapse legacy top-level destinations into the v3 primary surfaces so
    /// deep links and restored prefs keep working without a 14-item sidebar.
    fn normalize_navigation_for_v3(&mut self) {
        if let Some(tab) = NodeWorkspaceTab::from_legacy_view(self.selected_view) {
            if self.selected_view != View::Nodes {
                self.node_workspace_tab = tab;
                self.selected_view = View::Nodes;
            }
        }
        if self.selected_view == View::Snapshots {
            self.runtimes_section = RuntimesSection::Sync;
            self.selected_view = View::Runtimes;
        }
        if self.selected_view == View::Alerts {
            self.settings_section = SettingsSection::Alerts;
            self.selected_view = View::Settings;
        }
    }
}
