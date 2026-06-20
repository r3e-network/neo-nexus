use eframe::egui;

use crate::app::{widgets::labeled_text, NeoNexusApp};

pub(super) fn render_remote_profile_form(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    labeled_text(ui, "Name", &mut app.remote_server_name);
    labeled_text(ui, "Base URL", &mut app.remote_server_base_url);
    ui.horizontal(|ui| {
        ui.label("Description");
        ui.add_sized(
            [ui.available_width().max(120.0), 24.0],
            egui::TextEdit::singleline(&mut app.remote_server_description),
        );
    });
    ui.checkbox(&mut app.remote_server_enabled, "Enabled for manual probes");
}
