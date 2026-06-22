use eframe::egui;

use crate::app::{format_duration, widgets::metric_row, NeoNexusApp};

pub(super) fn render_settings_metrics(app: &NeoNexusApp, ui: &mut egui::Ui) {
    let policy = app.watchdog.policy();
    let attempts = policy.max_restart_attempts.to_string();
    let base_delay = format_duration(policy.base_delay);
    let max_delay = format_duration(policy.max_delay);
    metric_row(
        ui,
        &[
            (
                "Watchdog",
                if policy.enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
                "automatic restart",
            ),
            ("Attempts", &attempts, "per failure run"),
            ("Base Delay", &base_delay, "first retry"),
            ("Max Delay", &max_delay, "retry cap"),
        ],
    );
}
