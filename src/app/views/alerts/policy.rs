mod actions;
mod form;
mod preview;
mod status;
mod target;

use eframe::egui;

use crate::app::NeoNexusApp;

pub(super) fn render_alert_policy_editor(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    status::render_policy_status(ui, &app.async_bus.alert_routing_policy);
    ui.separator();
    form::render_policy_form(app, ui);
    target::render_target_editor(app, ui);
    actions::render_policy_actions(app, ui);
    preview::render_alert_preview(app, ui);
}
