use crate::app::theme;
use eframe::egui;

use crate::app::{domain::alert_target_label, theme::muted_text, NeoNexusApp};

pub(super) fn render_target_editor(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new(app.async_bus.alert_routing_policy_draft.provider.target_label())
            .color(muted_text()),
    );
    ui.add_sized(
        [ui.available_width().max(160.0), 24.0],
        egui::TextEdit::singleline(&mut app.async_bus.alert_routing_policy_draft.webhook_url)
            .password(true)
            .hint_text(app.async_bus.alert_routing_policy_draft.provider.target_hint()),
    );
    render_target_state(app, ui);
}

fn render_target_state(app: &NeoNexusApp, ui: &mut egui::Ui) {
    if let Some(message) = app.async_bus.alert_routing_policy_draft.validation_message() {
        ui.label(egui::RichText::new(message).color(theme::danger()));
    } else if !app.async_bus.alert_routing_policy_draft.webhook_url.trim().is_empty() {
        ui.label(
            egui::RichText::new(format!(
                "Target: {}",
                alert_target_label(&app.async_bus.alert_routing_policy_draft.webhook_url)
            ))
            .color(muted_text()),
        );
    } else {
        ui.label(egui::RichText::new("No outbound route configured.").color(muted_text()));
    }
}
