use eframe::egui;

use super::super::super::{theme::muted_text, widgets::chip_pill, NeoNexusApp};

pub(super) fn render_wallet_profile_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Usage").color(muted_text()));
        chip_pill(ui, |ui| {
            filter_button(app, ui, "All", None);
            filter_button(app, ui, "Used", Some(true));
            filter_button(app, ui, "Unused", Some(false));
        });
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.wallet_profile_query).hint_text("Search"),
    );
    if response.changed() {
        app.wallet_profile_page = 0;
    }
    ui.separator();
}

fn filter_button(app: &mut NeoNexusApp, ui: &mut egui::Ui, label: &str, used: Option<bool>) {
    if ui
        .selectable_label(app.wallet_profile_used_filter == used, label)
        .clicked()
    {
        app.wallet_profile_used_filter = used;
        app.wallet_profile_page = 0;
    }
}
