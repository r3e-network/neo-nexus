mod alerts;
mod config;
mod federation;
mod logs;
mod monitor;
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
pub(in crate::app) use operations::OperationsSection;
pub(in crate::app) use roles::RolesSection;
pub(in crate::app) use runtimes::RuntimesSection;
pub(in crate::app) use settings::SettingsSection;
pub(in crate::app) use snapshots::SnapshotsSection;

impl NeoNexusApp {
    pub(super) fn render_workspace(&mut self, ui: &mut egui::Ui) {
        ui.add_space(2.0);

        match self.selected_view {
            View::Summary => self.render_overview(ui),
            View::Operations => self.render_operations(ui),
            View::Monitor => self.render_monitor(ui),
            View::Alerts => self.render_alerts(ui),
            View::Federation => self.render_federation(ui),
            View::Settings => self.render_settings(ui),
            View::Runtimes => self.render_runtimes(ui),
            View::Wallets => self.render_wallets(ui),
            View::Nodes => self.render_nodes(ui),
            View::Roles => self.render_roles(ui),
            View::Snapshots => self.render_snapshots(ui),
            View::Plugins => self.render_plugins(ui),
            View::Config => self.render_config(ui),
            View::Logs => self.render_logs(ui),
        }
    }
}
