use eframe::egui;

use crate::app::{domain::RuntimeUpgradePolicy, NeoNexusApp};

pub(super) fn render_policy_actions(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    active_policy: &RuntimeUpgradePolicy,
) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let can_save = app
            .runtime_upgrade_policy_draft
            .validation_message()
            .is_none()
            && app.runtime_upgrade_policy_draft.differs_from(active_policy);
        if ui
            .add_enabled(can_save, egui::Button::new("Save Policy"))
            .clicked()
        {
            app.save_runtime_upgrade_policy();
        }
        if ui
            .add_enabled(
                app.runtime_upgrade_policy_draft.differs_from(active_policy),
                egui::Button::new("Reset Draft"),
            )
            .clicked()
        {
            app.reset_runtime_upgrade_policy_draft();
        }
    });
    if ui
        .add_enabled(active_policy.enabled, egui::Button::new("Run Policy Now"))
        .clicked()
    {
        app.run_runtime_upgrade_policy_now();
    }
}
