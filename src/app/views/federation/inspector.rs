use eframe::egui;

use super::super::super::{
    text::truncate_middle,
    theme::{muted_text, section_title},
    widgets::{empty_state, fact},
    NeoNexusApp,
};

mod colors;
mod history;
mod report;

use colors::remote_enabled_color;
use history::render_probe_history;
use report::render_probe_report;

impl NeoNexusApp {
    pub(in crate::app::views::federation) fn render_remote_profile_inspector(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        let Some(profile) = self.selected_remote_server_profile() else {
            empty_state(ui, "No selection", "Select a remote profile to inspect.");
            return;
        };

        ui.horizontal(|ui| {
            ui.label(section_title(truncate_middle(&profile.name, 28)).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(if profile.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    })
                    .strong()
                    .color(remote_enabled_color(profile.enabled)),
                );
            });
        });
        fact(ui, "Base URL", &truncate_middle(&profile.base_url, 46));
        fact(ui, "Created", &profile.created_at_unix.to_string());
        fact(ui, "Updated", &profile.updated_at_unix.to_string());
        fact(
            ui,
            "Description",
            &truncate_middle(&non_empty(&profile.description), 46),
        );
        ui.separator();
        fact(
            ui,
            "Status URL",
            &truncate_middle(&profile.public_status_url(), 52),
        );
        fact(
            ui,
            "Nodes URL",
            &truncate_middle(&profile.public_nodes_url(), 52),
        );
        fact(
            ui,
            "Metrics URL",
            &truncate_middle(&profile.public_system_metrics_url(), 52),
        );

        ui.separator();
        if let Some(report) = self.selected_remote_server_probe() {
            render_probe_report(ui, &report);
        } else {
            ui.label(
                egui::RichText::new("No probe result for selected profile.").color(muted_text()),
            );
        }
        ui.separator();
        render_probe_history(self, ui);
    }
}

fn non_empty(value: &str) -> String {
    if value.trim().is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}
