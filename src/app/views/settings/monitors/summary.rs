use eframe::egui;

use super::widgets::status_label;
use crate::app::{widgets::fact, NeoNexusApp};

pub(super) fn render_monitor_summary(app: &NeoNexusApp, ui: &mut egui::Ui) {
    ui.columns(2, |columns| {
        fact(
            &mut columns[0],
            "RPC",
            status_label(app.rpc_health_monitor_policy.enabled),
        );
        fact(
            &mut columns[0],
            "RPC pending",
            &app.rpc_health_pending.len().to_string(),
        );
        fact(
            &mut columns[1],
            "Federation",
            status_label(app.remote_federation_monitor_policy.enabled),
        );
        fact(
            &mut columns[1],
            "Fed pending",
            &app.remote_federation_pending.len().to_string(),
        );
    });
}
