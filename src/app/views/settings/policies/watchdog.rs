use eframe::egui;

use super::super::super::super::{
    theme::{accent, muted_text},
    widgets::fact,
    NeoNexusApp,
};

impl NeoNexusApp {
    pub(in crate::app::views::settings) fn render_watchdog_policy_settings(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        let active_policy = self.watchdog.policy();

        ui.checkbox(
            &mut self.watchdog_policy_draft.enabled,
            "Enable automatic restart after abnormal process exits",
        );
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Max attempts");
            ui.add(
                egui::DragValue::new(&mut self.watchdog_policy_draft.max_restart_attempts)
                    .range(0..=20)
                    .speed(1.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Base delay");
            ui.add(
                egui::DragValue::new(&mut self.watchdog_policy_draft.base_delay_seconds)
                    .range(1..=3_600)
                    .suffix(" s")
                    .speed(1.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Max delay");
            ui.add(
                egui::DragValue::new(&mut self.watchdog_policy_draft.max_delay_seconds)
                    .range(1..=86_400)
                    .suffix(" s")
                    .speed(1.0),
            );
        });

        ui.add_space(6.0);
        if let Some(message) = self.watchdog_policy_draft.validation_message() {
            ui.label(egui::RichText::new(message).color(egui::Color32::from_rgb(185, 28, 28)));
        } else {
            ui.label(egui::RichText::new("Policy draft is valid.").color(accent()));
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let can_save = self.watchdog_policy_draft.validation_message().is_none()
                && self.watchdog_policy_draft.differs_from(active_policy);
            if ui
                .add_enabled(can_save, egui::Button::new("Save Policy"))
                .clicked()
            {
                self.save_watchdog_policy();
            }
            if ui
                .add_enabled(
                    self.watchdog_policy_draft.differs_from(active_policy),
                    egui::Button::new("Reset Draft"),
                )
                .clicked()
            {
                self.reset_watchdog_policy_draft();
            }
        });

        ui.separator();
        fact(ui, "Active", &active_policy.describe());
        fact(
            ui,
            "Pending",
            if self.watchdog.has_pending_restart() {
                "yes"
            } else {
                "no"
            },
        );
        ui.label(
            egui::RichText::new(
                "Saving clears pending watchdog timers and applies the new policy.",
            )
            .color(muted_text()),
        );
    }
}
