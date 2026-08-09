use eframe::egui;

use crate::app::{
    domain::RuntimeUpgradePolicy,
    theme::muted_text,
    widgets::{fact, hr_tight},
};

use super::super::time::time_fact;

pub(super) fn render_policy_status(ui: &mut egui::Ui, active_policy: &RuntimeUpgradePolicy) {
    hr_tight(ui);
    // Two columns: four one-line facts stacked cost four rows of a panel that
    // does not scroll, and each value is short enough that half the width is
    // more than it needs.
    ui.columns(2, |columns| {
        fact(&mut columns[0], "Active", &active_policy.describe());
        fact(
            &mut columns[0],
            "Window",
            &active_policy.maintenance_window_label(),
        );
        fact(
            &mut columns[1],
            "Last check",
            &time_fact(active_policy.last_checked_at_unix),
        );
        fact(
            &mut columns[1],
            "Last apply",
            &time_fact(active_policy.last_applied_at_unix),
        );
    });
    ui.label(
        egui::RichText::new(
            "Runs stopped-node upgrades directly and rolls running nodes through restart readiness.",
        )
        .color(muted_text()),
    );
}
