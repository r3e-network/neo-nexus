use eframe::egui;

use super::super::super::super::{
    format_duration,
    text::short_path,
    theme,
    widgets::{card, metric_grid},
    NeoNexusApp,
};

impl NeoNexusApp {
    pub(super) fn render_runtime_facts(&self, ui: &mut egui::Ui) {
        card(ui, "Runtime", Some("pure Rust"), |ui| {
            let policy = self.watchdog.policy();
            let watchdog = if policy.enabled {
                format!(
                    "{} × {}",
                    policy.max_restart_attempts,
                    format_duration(policy.base_delay)
                )
            } else {
                "disabled".to_string()
            };
            metric_grid(
                ui,
                &[
                    ("Interface", "egui / eframe".to_string()),
                    ("Build", env!("CARGO_PKG_VERSION").to_string()),
                    ("Watchdog", watchdog),
                    ("Retry cap", format_duration(policy.max_delay)),
                    ("Workspace", short_path(self.repository.db_path(), 30)),
                ],
            );
            ui.add_space(theme::XS);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.label(theme::muted_body(
                "No browser wrapper, embedded runtime, or JS toolchain.",
            ));
        });
    }
}
