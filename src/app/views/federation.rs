use eframe::egui;

use super::super::{
    text::truncate_middle,
    theme,
    widgets::{metric_row, panel, segmented_control},
    NeoNexusApp,
};

mod inspector;
mod profiles;
mod section;

pub(in crate::app) use section::FederationSection;

impl NeoNexusApp {
    pub(super) fn render_federation(&mut self, ui: &mut egui::Ui) {
        let enabled = self
            .remote_servers
            .iter()
            .filter(|profile| profile.enabled)
            .count();
        let disabled = self.remote_servers.len().saturating_sub(enabled);
        let selected = self
            .selected_remote_server_profile()
            .map_or_else(|| "none".to_string(), |profile| profile.name);
        let probe = self
            .selected_remote_server_probe()
            .map_or("not probed", |report| report.status.label());
        let auto_label = if self.async_bus.remote_federation_monitor_policy.enabled {
            "enabled"
        } else {
            "disabled"
        };
        let auto_detail = format!("{} pending", self.async_bus.remote_federation_pending.len());

        let remotes = self.remote_servers.len().to_string();
        let enabled_label = enabled.to_string();
        let disabled_label = disabled.to_string();
        let selected_short = truncate_middle(&selected, 20);
        metric_row(
            ui,
            &[
                ("Remotes", &remotes, "saved profiles"),
                ("Enabled", &enabled_label, "active probes"),
                ("Disabled", &disabled_label, "paused profiles"),
                ("Auto", auto_label, &auto_detail),
                ("Probe", probe, &selected_short),
            ],
        );

        ui.add_space(theme::MD);
        let mut index = self.sections.federation as usize;
        let labels = FederationSection::ALL.map(FederationSection::label);
        if segmented_control(ui, &labels, &mut index) {
            self.sections.federation = FederationSection::ALL[index];
        }
        ui.add_space(theme::MD);

        match self.sections.federation {
            FederationSection::Profiles => panel(ui, "Remote profiles", |ui| {
                self.render_remote_profile_list(ui);
            }),
            FederationSection::Editor => panel(ui, "Profile editor", |ui| {
                self.render_remote_profile_editor(ui);
            }),
            FederationSection::Inspector => panel(ui, "Endpoint inspector", |ui| {
                self.render_remote_profile_inspector(ui);
            }),
        }
    }
}
