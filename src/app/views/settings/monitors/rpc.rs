use eframe::egui;

use super::widgets::{interval_drag, validation_error};
use crate::app::{domain::RpcHealthMonitorPolicy, NeoNexusApp};

pub(super) fn render_rpc_monitor_policy(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.checkbox(
        &mut app.async_bus.rpc_health_monitor_policy_draft.enabled,
        "Automatically check RPC health for running nodes",
    );
    interval_drag(
        ui,
        "RPC interval",
        &mut app.async_bus.rpc_health_monitor_policy_draft.interval_seconds,
        RpcHealthMonitorPolicy::MIN_INTERVAL_SECONDS..=RpcHealthMonitorPolicy::MAX_INTERVAL_SECONDS,
        5.0,
    );

    validation_error(ui, app.async_bus.rpc_health_monitor_policy_draft.validation_message());
    render_rpc_actions(app, ui);
}

fn render_rpc_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let can_save = app.async_bus
            .rpc_health_monitor_policy_draft
            .validation_message()
            .is_none()
            && app.async_bus
                .rpc_health_monitor_policy_draft
                .differs_from(app.async_bus.rpc_health_monitor_policy);
        if ui
            .add_enabled(can_save, egui::Button::new("Save RPC Monitor"))
            .clicked()
        {
            app.save_rpc_health_monitor_policy();
        }
        if ui
            .add_enabled(
                app.async_bus.rpc_health_monitor_policy_draft
                    .differs_from(app.async_bus.rpc_health_monitor_policy),
                egui::Button::new("Reset Draft"),
            )
            .clicked()
        {
            app.reset_rpc_health_monitor_policy_draft();
        }
    });
}
