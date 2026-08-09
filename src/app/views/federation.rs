use eframe::egui;

use super::super::{
    widgets::{page_chrome, panel},
    NeoNexusApp,
};

mod governance;
mod inspector;
mod profiles;
mod section;

pub(in crate::app) use section::FederationSection;

impl NeoNexusApp {
    pub(super) fn render_federation(&mut self, ui: &mut egui::Ui) {
        // No page metric row. Remotes, Enabled and Disabled count the profile
        // list, so they sit in the Profiles section with it; Auto is the
        // federation monitor, now grouped with the other monitors under
        // Monitor > Telemetry; and Probe is what the Inspector section shows in
        // full. Above the tabs they were ~90pt of restatement on every surface.
        let mut index = self.sections.federation as usize;
        let labels = FederationSection::ALL.map(FederationSection::label);
        if page_chrome(ui, None, Some((&labels, &mut index))) {
            self.sections.federation = FederationSection::ALL[index];
        }

        match self.sections.federation {
            FederationSection::Profiles => panel(ui, "Remote profiles", |ui| {
                self.render_remote_profile_counts(ui);
                self.render_remote_profile_list(ui);
            }),
            FederationSection::Editor => panel(ui, "Profile editor", |ui| {
                self.render_remote_profile_editor(ui);
            }),
            FederationSection::Governance => panel(ui, "Governance", |ui| {
                self.render_governance(ui);
            }),
            FederationSection::Inspector => panel(ui, "Endpoint inspector", |ui| {
                self.render_remote_profile_inspector(ui);
            }),
        }
    }
}
