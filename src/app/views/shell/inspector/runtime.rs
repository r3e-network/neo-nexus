use eframe::egui;

use super::super::super::super::{
    format_duration, text::short_path, theme::accent, widgets::fact, NeoNexusApp,
};

impl NeoNexusApp {
    pub(super) fn render_runtime_facts(&self, ui: &mut egui::Ui) {
        ui.strong("Runtime");
        fact(ui, "Application", "Pure Rust native");
        fact(ui, "GUI", "egui / eframe");
        fact(ui, "Build", env!("CARGO_PKG_VERSION"));
        fact(ui, "Storage", &short_path(self.repository.db_path(), 46));
        let policy = self.watchdog.policy();
        fact(
            ui,
            "Watchdog",
            &format!(
                "{}; {} base, {} cap",
                if policy.enabled {
                    format!("{} attempts", policy.max_restart_attempts)
                } else {
                    "disabled".to_string()
                },
                format_duration(policy.base_delay),
                format_duration(policy.max_delay)
            ),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("No browser wrapper, embedded runtime, or JS toolchain.")
                .color(accent()),
        );
    }
}
