use eframe::egui;

use crate::app::{
    sidecar_health::sidecar_execution_policy_label, text::format_optional_unix_age, widgets::fact,
    workflow::current_unix_time, NeoNexusApp,
};

pub(in crate::app::views::roles::private_network) fn render_sidecar_status(
    app: &NeoNexusApp,
    ui: &mut egui::Ui,
) {
    if let Some(report) = &app.private_network_sidecar_report {
        fact(
            ui,
            "Sidecars",
            &format!(
                "{} loaded, {} running",
                report.sidecar_count,
                app.private_network_sidecar_pids.len()
            ),
        );
        fact(ui, "Sidecar watchdog", &app.watchdog.policy().describe());
        fact(
            ui,
            "Sidecar policy",
            sidecar_execution_policy_label(app.private_network_allow_external_sidecars),
        );
        render_sidecar_health(app, ui);
    } else {
        fact(ui, "Sidecars", "not loaded");
    }
}

fn render_sidecar_health(app: &NeoNexusApp, ui: &mut egui::Ui) {
    if let Some(health) = &app.private_network_sidecar_health_report {
        fact(
            ui,
            "Sidecar health",
            &format!(
                "{}/{} reachable, {} missing",
                health.reachable_count, health.endpoint_count, health.missing_endpoint_count
            ),
        );
        fact(
            ui,
            "Health checked",
            &health_checked_label(health.checked_at_unix),
        );
    } else {
        fact(ui, "Sidecar health", "not checked");
    }
}

fn health_checked_label(checked_at_unix: u64) -> String {
    current_unix_time().map_or_else(
        |_| format!("unix {checked_at_unix}"),
        |now| format_optional_unix_age(Some(checked_at_unix), now),
    )
}
