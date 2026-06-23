use eframe::egui;

use crate::app::{theme::muted_text, widgets::chip_pill, NeoNexusApp};

pub(super) fn render_remote_profile_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Enabled").color(muted_text()));
        chip_pill(ui, |ui| {
            filter_button(app, ui, "All", None);
            filter_button(app, ui, "On", Some(true));
            filter_button(app, ui, "Off", Some(false));
        });
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.remote_server_query).hint_text("Search"),
    );
    if response.changed() {
        app.remote_server_page = 0;
    }
    ui.separator();
}

fn filter_button(app: &mut NeoNexusApp, ui: &mut egui::Ui, label: &str, enabled: Option<bool>) {
    if ui
        .selectable_label(app.remote_server_enabled_filter == enabled, label)
        .clicked()
    {
        app.remote_server_enabled_filter = enabled;
        app.remote_server_page = 0;
    }
}
