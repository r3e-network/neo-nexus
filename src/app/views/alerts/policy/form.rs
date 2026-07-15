use eframe::egui;

use crate::app::{
    domain::{AlertProvider, AlertRoutingPolicy, EventSeverity},
    widgets::labeled_combo,
    NeoNexusApp,
};

pub(super) fn render_policy_form(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.checkbox(
        &mut app.async_bus.alert_routing_policy_draft.enabled,
        "Enable webhook alert routing",
    );
    render_provider_picker(app, ui);
    render_severity_picker(app, ui);
    render_timeout_editor(app, ui);
}

fn render_provider_picker(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    labeled_combo(
        ui,
        "Provider",
        "alert_provider",
        app.async_bus.alert_routing_policy_draft
            .provider
            .display_name()
            .to_string(),
        |ui| {
            for provider in AlertProvider::ALL {
                ui.selectable_value(
                    &mut app.async_bus.alert_routing_policy_draft.provider,
                    provider,
                    provider.display_name(),
                );
            }
        },
    );
}

fn render_severity_picker(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    labeled_combo(
        ui,
        "Minimum severity",
        "alert_min_severity",
        app.async_bus.alert_routing_policy_draft.min_severity.to_string(),
        |ui| {
            for severity in EventSeverity::ALL {
                ui.selectable_value(
                    &mut app.async_bus.alert_routing_policy_draft.min_severity,
                    severity,
                    severity.label(),
                );
            }
        },
    );
}

fn render_timeout_editor(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Timeout");
        ui.add(
            egui::DragValue::new(&mut app.async_bus.alert_routing_policy_draft.timeout_seconds)
                .range(
                    AlertRoutingPolicy::MIN_TIMEOUT_SECONDS
                        ..=AlertRoutingPolicy::MAX_TIMEOUT_SECONDS,
                )
                .suffix(" s")
                .speed(1.0),
        );
    });
}
