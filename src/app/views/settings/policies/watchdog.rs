use eframe::egui;

use crate::app::theme;

use super::super::super::super::{
    widgets::{callout, fact, form_group, primary_button, secondary_button_enabled, CalloutKind},
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
        ui.add_space(theme::MD);

        form_group(ui, "Restart limits", |ui| {
            ui.horizontal(|ui| {
                ui.label(theme::muted_body("Max attempts"));
                ui.add(
                    egui::DragValue::new(&mut self.watchdog_policy_draft.max_restart_attempts)
                        .range(0..=20)
                        .speed(1.0),
                );
            });
            ui.add_space(theme::SM);
            ui.horizontal(|ui| {
                ui.label(theme::muted_body("Base delay"));
                ui.add(
                    egui::DragValue::new(&mut self.watchdog_policy_draft.base_delay_seconds)
                        .range(1..=3_600)
                        .suffix(" s")
                        .speed(1.0),
                );
            });
            ui.add_space(theme::SM);
            ui.horizontal(|ui| {
                ui.label(theme::muted_body("Max delay"));
                ui.add(
                    egui::DragValue::new(&mut self.watchdog_policy_draft.max_delay_seconds)
                        .range(1..=86_400)
                        .suffix(" s")
                        .speed(1.0),
                );
            });
        });

        ui.add_space(theme::MD);
        if let Some(message) = self.watchdog_policy_draft.validation_message() {
            callout(ui, CalloutKind::Danger, "Invalid draft", message);
        } else {
            callout(
                ui,
                CalloutKind::Success,
                "Draft is valid",
                "Save to apply the policy to the supervisor.",
            );
        }

        ui.add_space(theme::MD);
        let can_save = self.watchdog_policy_draft.validation_message().is_none()
            && self.watchdog_policy_draft.differs_from(active_policy);
        let can_reset = self.watchdog_policy_draft.differs_from(active_policy);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(can_save, |ui| {
                if primary_button(ui, "Save Policy").clicked() {
                    self.save_watchdog_policy();
                }
            });
            if secondary_button_enabled(ui, "Reset Draft", can_reset).clicked() {
                self.reset_watchdog_policy_draft();
            }
        });

        ui.add_space(theme::MD);
        ui.separator();
        ui.add_space(theme::SM);
        fact(ui, "Active", &active_policy.describe());
    }
}
