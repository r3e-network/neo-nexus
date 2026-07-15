use super::*;

pub(in crate::app::views::roles::private_network) fn render_signer_inputs(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
) {
    ui.add_space(theme::SM);
    labeled_text(ui, "Committee", &mut app.private_network_committee_keys);
    ui.label(
        egui::RichText::new("Compressed public keys, comma or space separated.")
            .color(muted_text()),
    );
    ui.horizontal(|ui| {
        ui.label("Signers");
        ui.add_sized(
            [(ui.available_width() - 112.0).max(180.0), 54.0],
            egui::TextEdit::multiline(&mut app.private_network_signer_refs)
                .desired_rows(2)
                .hint_text(
                    "public_key | wallet path | http(s) endpoint | sidecar command template",
                ),
        );
        if ui
            .add_enabled(
                app.selected_neo_wallet_profile().is_some(),
                egui::Button::new("Use Wallet"),
            )
            .on_hover_text("Add the selected Wallet Profile as a signer reference")
            .clicked()
        {
            app.use_selected_neo_wallet_profile_for_private_network_signer_refs();
        }
    });
    ui.label(
        egui::RichText::new(
            "References and commands only: launch packs add wallet provisioning evidence but never include private keys or passwords.",
        )
        .color(muted_text()),
    );
}
