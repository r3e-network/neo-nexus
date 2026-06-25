use eframe::egui;

use crate::app::{domain::RuntimeUpgradePolicy, theme, widgets, NeoNexusApp};

pub(super) fn render_policy_actions(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    active_policy: &RuntimeUpgradePolicy,
) {
    ui.add_space(theme::SM);
    ui.horizontal(|ui| {
        let can_save = app
            .runtime_upgrade_policy_draft
            .validation_message()
            .is_none()
            && app.runtime_upgrade_policy_draft.differs_from(active_policy);
        if widgets::secondary_button_enabled(ui, "Save Policy", can_save).clicked() {
            app.save_runtime_upgrade_policy();
        }
        if widgets::secondary_button_enabled(
            ui,
            "Reset Draft",
            app.runtime_upgrade_policy_draft.differs_from(active_policy),
        )
        .clicked()
        {
            app.reset_runtime_upgrade_policy_draft();
        }
        if widgets::secondary_button_enabled(ui, "Run Policy Now", active_policy.enabled).clicked()
        {
            app.run_runtime_upgrade_policy_now();
        }
    });
}
