use eframe::egui;

use crate::app::{domain::RuntimeUpgradePolicy, theme::muted_text, widgets::fact};

use super::super::time::time_fact;

pub(super) fn render_policy_status(ui: &mut egui::Ui, active_policy: &RuntimeUpgradePolicy) {
    ui.separator();
    fact(ui, "Active", &active_policy.describe());
    fact(
        ui,
        "Last check",
        &time_fact(active_policy.last_checked_at_unix),
    );
    fact(ui, "Window", &active_policy.maintenance_window_label());
    fact(
        ui,
        "Last apply",
        &time_fact(active_policy.last_applied_at_unix),
    );
    ui.label(
        egui::RichText::new(
            "Runs stopped-node upgrades directly and rolls running nodes through restart readiness.",
        )
        .color(muted_text()),
    );
}
