use eframe::egui;

use crate::app::{
    theme,
    widgets::{page_chrome, panel},
    NeoNexusApp,
};

use super::{metrics::render_settings_metrics, SettingsSection};

pub(super) fn render_settings(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    render_settings_metrics(app, ui);

    ui.add_space(theme::MD);
    let mut index = app.sections.settings as usize;
    let labels = SettingsSection::ALL.map(SettingsSection::label);
    if page_chrome(ui, None, Some((&labels, &mut index))) {
        app.sections.settings = SettingsSection::ALL[index];
    }

    match app.sections.settings {
        SettingsSection::Watchdog => panel(ui, "Watchdog policy", |ui| {
            app.render_watchdog_policy_settings(ui);
        }),
        SettingsSection::Upgrades => panel(ui, "Runtime upgrade policy", |ui| {
            app.render_runtime_upgrade_policy_settings(ui);
        }),
        SettingsSection::Monitors => panel(ui, "Health monitors", |ui| {
            app.render_rpc_monitor_settings(ui);
        }),
        SettingsSection::Alerts => app.render_alerts(ui),
        SettingsSection::Storage => panel(ui, "Workspace storage", |ui| {
            app.render_workspace_storage_settings(ui);
        }),
        SettingsSection::Release => panel(ui, "Release handoff", |ui| {
            app.render_release_package_settings(ui);
        }),
    }
}
