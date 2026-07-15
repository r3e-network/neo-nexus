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
        let draft_differs = self.watchdog_policy_draft.differs_from(active_policy);
        let validation = self.watchdog_policy_draft.validation_message();

        form_group(ui, "Policy", |ui| {
            ui.checkbox(
                &mut self.watchdog_policy_draft.enabled,
                "Enable automatic restart after abnormal process exits",
            );
        });
        ui.add_space(theme::MD);

        form_group(ui, "Restart limits", |ui| {
            ui.horizontal(|ui| {
                ui.set_min_width(120.0);
                ui.label(theme::muted_body("Max attempts"));
                ui.add(
                    egui::DragValue::new(&mut self.watchdog_policy_draft.max_restart_attempts)
                        .range(0..=20)
                        .speed(1.0),
                );
            });
            ui.add_space(theme::SM);
            ui.horizontal(|ui| {
                ui.set_min_width(120.0);
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
                ui.set_min_width(120.0);
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
        if let Some(message) = validation {
            callout(ui, CalloutKind::Danger, "Invalid draft", message);
        } else if draft_differs {
            callout(
                ui,
                CalloutKind::Info,
                "Unsaved changes",
                "Save to apply this draft to the supervisor.",
            );
        } else {
            callout(
                ui,
                CalloutKind::Success,
                "In sync",
                "Draft matches the active watchdog policy.",
            );
        }

        ui.add_space(theme::MD);
        let can_save = validation.is_none() && draft_differs;
        ui.horizontal(|ui| {
            ui.add_enabled_ui(can_save, |ui| {
                if primary_button(ui, "Save Policy")
                    .on_hover_text("Persist and activate the draft policy")
                    .clicked()
                {
                    self.save_watchdog_policy();
                }
            });
            if secondary_button_enabled(ui, "Reset Draft", draft_differs)
                .on_hover_text("Discard draft edits and reload the active policy")
                .clicked()
            {
                self.reset_watchdog_policy_draft();
            }
        });

        ui.add_space(theme::MD);
        ui.separator();
        ui.add_space(theme::SM);
        ui.label(theme::label_caption("Active policy"));
        ui.add_space(theme::XS);
        fact(ui, "Description", &active_policy.describe());
    }
}
