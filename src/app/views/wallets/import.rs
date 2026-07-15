use eframe::egui;

use super::super::super::{
    theme::accent,
    widgets::{fact, labeled_text, primary_button},
    NeoNexusApp,
};

pub(super) fn render_wallet_profile_import_form(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    labeled_text(ui, "Wallet path", &mut app.wallet_profile_source);
    labeled_text(ui, "Profile ID", &mut app.wallet_profile_id);
    labeled_text(ui, "Label", &mut app.wallet_profile_label);
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if primary_button(ui, "Import").clicked() {
            app.import_neo_wallet_profile_from_form();
        }
        if ui.button("Reset").clicked() {
            reset_wallet_profile_form(app);
        }
    });

    ui.separator();
    ui.strong("Storage boundary");
    fact(ui, "Material", "metadata only");
    fact(ui, "Secrets", "not stored");
    fact(ui, "Wallet bytes", "not copied");
    fact(ui, "Hash", "SHA-256 retained");
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("NEP-6 encrypted wallet validation runs before persistence.")
            .color(accent()),
    );
}

fn reset_wallet_profile_form(app: &mut NeoNexusApp) {
    app.wallet_profile_source.clear();
    app.wallet_profile_id = "validator-wallet".to_string();
    app.wallet_profile_label = "Validator wallet".to_string();
    app.session.notice = Some("Wallet profile form reset".to_string());
}
