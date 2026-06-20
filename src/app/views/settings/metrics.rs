use eframe::egui;

use crate::app::{format_duration, widgets::metric_tile, NeoNexusApp};

pub(super) fn render_settings_metrics(app: &NeoNexusApp, ui: &mut egui::Ui) {
    let policy = app.watchdog.policy();
    ui.horizontal(|ui| {
        metric_tile(
            ui,
            "Watchdog",
            if policy.enabled {
                "Enabled"
            } else {
                "Disabled"
            },
            "automatic restart",
        );
        metric_tile(
            ui,
            "Attempts",
            &policy.max_restart_attempts.to_string(),
            "per failure run",
        );
        metric_tile(
            ui,
            "Base Delay",
            &format_duration(policy.base_delay),
            "first retry",
        );
        metric_tile(
            ui,
            "Max Delay",
            &format_duration(policy.max_delay),
            "retry cap",
        );
    });
}
