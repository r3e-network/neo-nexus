use eframe::egui;

use super::widgets::{interval_drag, validation_error};
use crate::{app::NeoNexusApp, federation::RemoteFederationMonitorPolicy};

pub(super) fn render_federation_monitor_policy(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.checkbox(
        &mut app.remote_federation_monitor_policy_draft.enabled,
        "Automatically probe enabled remote Federation profiles",
    );
    interval_drag(
        ui,
        "Federation interval",
        &mut app.remote_federation_monitor_policy_draft.interval_seconds,
        RemoteFederationMonitorPolicy::MIN_INTERVAL_SECONDS
            ..=RemoteFederationMonitorPolicy::MAX_INTERVAL_SECONDS,
        10.0,
    );

    validation_error(
        ui,
        app.remote_federation_monitor_policy_draft
            .validation_message(),
    );
    render_federation_actions(app, ui);
}

fn render_federation_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let can_save = app
            .remote_federation_monitor_policy_draft
            .validation_message()
            .is_none()
            && app
                .remote_federation_monitor_policy_draft
                .differs_from(app.remote_federation_monitor_policy);
        if ui
            .add_enabled(can_save, egui::Button::new("Save Federation"))
            .clicked()
        {
            app.save_remote_federation_monitor_policy();
        }
        if ui
            .add_enabled(
                app.remote_federation_monitor_policy_draft
                    .differs_from(app.remote_federation_monitor_policy),
                egui::Button::new("Reset Draft"),
            )
            .clicked()
        {
            app.reset_remote_federation_monitor_policy_draft();
        }
    });
}
