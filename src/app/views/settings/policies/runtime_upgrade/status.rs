use eframe::egui;

use crate::app::{domain::RuntimeUpgradePolicy, theme::muted_text, widgets::hr_tight};

use super::super::time::time_fact;

pub(super) fn render_policy_status(ui: &mut egui::Ui, active_policy: &RuntimeUpgradePolicy) {
    hr_tight(ui);
    // A grid, not `fact`: `fact` right-aligns its value across whatever width it
    // is given, so on a full-width settings panel each label sat at one edge of
    // the workspace and its value at the other. Here the value follows its label,
    // and the four of them pair up into two rows instead of four.
    egui::Grid::new("runtime_upgrade_policy_status")
        .num_columns(4)
        .spacing([18.0, 4.0])
        .show(ui, |ui| {
            status_pair(ui, "Active", &active_policy.describe());
            status_pair(
                ui,
                "Last check",
                &time_fact(active_policy.last_checked_at_unix),
            );
            ui.end_row();
            status_pair(ui, "Window", &active_policy.maintenance_window_label());
            status_pair(
                ui,
                "Last apply",
                &time_fact(active_policy.last_applied_at_unix),
            );
            ui.end_row();
        });
    ui.label(
        egui::RichText::new(
            "Runs stopped-node upgrades directly and rolls running nodes through restart readiness.",
        )
        .color(muted_text()),
    );
}

fn status_pair(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(egui::RichText::new(label).color(muted_text()));
    ui.label(egui::RichText::new(value).strong());
}
