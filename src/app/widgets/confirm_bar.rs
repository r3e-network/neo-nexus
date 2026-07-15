//! Destructive confirmation block: danger callout + danger-filled confirm + cancel.

use eframe::egui;

use crate::app::{
    theme,
    widgets::{callout, secondary_button, CalloutKind},
};

/// Renders a destructive confirmation. Returns `Some(true)` if confirm clicked,
/// `Some(false)` if cancel clicked, `None` if neither.
pub(in crate::app) fn confirm_bar(
    ui: &mut egui::Ui,
    title: &str,
    body: &str,
    confirm_label: &str,
    cancel_label: &str,
    confirm_enabled: bool,
) -> Option<bool> {
    ui.add_space(theme::MD);
    callout(ui, CalloutKind::Danger, title, body);
    ui.add_space(theme::SM);
    let mut outcome = None;
    ui.horizontal(|ui| {
        ui.add_enabled_ui(confirm_enabled, |ui| {
            let response = ui.add(
                egui::Button::new(
                    egui::RichText::new(confirm_label)
                        .color(theme::on_accent())
                        .strong(),
                )
                .fill(theme::danger())
                .corner_radius(egui::CornerRadius::same(8))
                .min_size(egui::vec2(100.0, 30.0)),
            );
            if response.clicked() {
                outcome = Some(true);
            }
        });
        if secondary_button(ui, cancel_label).clicked() {
            outcome = Some(false);
        }
    });
    outcome
}
