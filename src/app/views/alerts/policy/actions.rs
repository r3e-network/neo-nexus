use eframe::egui;

use crate::app::{theme::muted_text, widgets, NeoNexusApp};

pub(super) fn render_policy_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.separator();
    ui.horizontal(|ui| {
        if widgets::secondary_button_enabled(ui, "Save Route", can_save_policy(app)).clicked() {
            app.save_alert_routing_policy();
        }
        if widgets::secondary_button_enabled(ui, "Reset Draft", can_reset_policy(app)).clicked() {
            app.reset_alert_routing_policy_draft();
        }
        if widgets::secondary_button_enabled(ui, "Preview Route", can_preview_policy(app)).clicked()
        {
            app.preview_alert_routing_policy_draft();
        }
        if widgets::secondary_button(ui, "Prune History").clicked() {
            app.prune_alert_delivery_history();
        }
    });
    ui.label(
        egui::RichText::new("Webhook secrets stay local and are not included in JSON backups.")
            .color(muted_text()),
    );
}

fn can_save_policy(app: &NeoNexusApp) -> bool {
    app.alert_routing_policy_draft
        .validation_message()
        .is_none()
        && can_reset_policy(app)
}

fn can_reset_policy(app: &NeoNexusApp) -> bool {
    app.alert_routing_policy_draft
        .differs_from(&app.alert_routing_policy)
}

fn can_preview_policy(app: &NeoNexusApp) -> bool {
    app.alert_routing_policy_draft
        .validation_message()
        .is_none()
        && !app.alert_routing_policy_draft.webhook_url.trim().is_empty()
}
