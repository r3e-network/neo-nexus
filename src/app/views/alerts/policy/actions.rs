use eframe::egui;

use crate::app::{theme::muted_text, widgets, widgets::hr_tight, NeoNexusApp};

/// The three things an operator can do to the draft.
///
/// `horizontal_wrapped` rather than `horizontal`: four buttons in a row needed
/// ~400pt in a pane that is 358pt wide, and the fourth painted outside the column
/// — which is also what pushed the whole Alerts page 46pt past its clip. Prune
/// History moved to the Delivery history panel, where the history it prunes is.
pub(super) fn render_policy_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    hr_tight(ui);
    ui.horizontal_wrapped(|ui| {
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
    });
    // One line, not two: at 358pt the longer wording wrapped, and a wrapped
    // static note costs the same vertical room as a row of real content.
    ui.label(egui::RichText::new("Secrets stay local — never in backups.").color(muted_text()));
}

fn can_save_policy(app: &NeoNexusApp) -> bool {
    app.async_bus
        .alert_routing_policy_draft
        .validation_message()
        .is_none()
        && can_reset_policy(app)
}

fn can_reset_policy(app: &NeoNexusApp) -> bool {
    app.async_bus
        .alert_routing_policy_draft
        .differs_from(&app.async_bus.alert_routing_policy)
}

fn can_preview_policy(app: &NeoNexusApp) -> bool {
    app.async_bus
        .alert_routing_policy_draft
        .validation_message()
        .is_none()
        && !app
            .async_bus
            .alert_routing_policy_draft
            .webhook_url
            .trim()
            .is_empty()
}
