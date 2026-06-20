use eframe::egui;

use crate::app::{theme::muted_text, NeoNexusApp};

pub(super) fn render_policy_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.separator();
    ui.horizontal(|ui| {
        if ui
            .add_enabled(can_save_policy(app), egui::Button::new("Save Route"))
            .clicked()
        {
            app.save_alert_routing_policy();
        }
        if ui
            .add_enabled(can_reset_policy(app), egui::Button::new("Reset Draft"))
            .clicked()
        {
            app.reset_alert_routing_policy_draft();
        }
        if ui
            .add_enabled(can_preview_policy(app), egui::Button::new("Preview Route"))
            .clicked()
        {
            app.preview_alert_routing_policy_draft();
        }
        if ui.button("Prune History").clicked() {
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
