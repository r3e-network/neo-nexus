use std::path::Path;

use eframe::egui;

use super::super::super::{
    text::{short_path, truncate_middle},
    theme::{self, accent, muted_text},
    widgets::{empty_state, fact, secondary_button},
    NeoNexusApp,
};

pub(super) fn render_selected_wallet_profile(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let Some(profile) = app.selected_neo_wallet_profile() else {
        empty_state(
            ui,
            "No wallet selected",
            "Select a profile from the registry.",
        );
        return;
    };

    ui.horizontal(|ui| {
        ui.label(theme::section_title(truncate_middle(&profile.label, 30)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new("metadata").color(accent()).strong());
        });
    });
    fact(ui, "Profile ID", &truncate_middle(&profile.id, 46));
    fact(ui, "Address", &profile.primary_address);
    fact(
        ui,
        "Source",
        &short_path(Path::new(&profile.source_path), 46),
    );
    fact(
        ui,
        "Wallet",
        profile.wallet_version.as_deref().unwrap_or("-"),
    );
    fact(ui, "Accounts", &profile.account_count.to_string());
    fact(
        ui,
        "Encrypted",
        &profile.encrypted_account_count.to_string(),
    );
    fact(ui, "Default", &profile.default_account_count.to_string());
    fact(
        ui,
        "Watch-only",
        &profile.watch_only_account_count.to_string(),
    );
    fact(ui, "Hash", &truncate_middle(&profile.wallet_sha256, 46));
    fact(
        ui,
        "Validated",
        &format_optional_unix(Some(profile.validated_at_unix)),
    );
    fact(
        ui,
        "Last used",
        &format_optional_unix(profile.last_used_at_unix),
    );

    ui.separator();
    ui.label(egui::RichText::new("Contract public keys").color(muted_text()));
    for index in 0..3 {
        let value = profile
            .contract_public_keys
            .get(index)
            .map_or("-", String::as_str);
        fact(
            ui,
            &format!("Key {}", index + 1),
            &truncate_middle(value, 46),
        );
    }

    ui.add_space(theme::SM);
    ui.horizontal(|ui| {
        if secondary_button(ui, "Use").clicked() {
            app.mark_selected_neo_wallet_profile_used();
        }
        if secondary_button(ui, "Use in Roles").clicked() {
            app.use_selected_neo_wallet_profile_for_private_network_signer_refs();
        }
        if secondary_button(ui, "Load Form").clicked() {
            app.wallet_profile_source = profile.source_path.clone();
            app.wallet_profile_id = profile.id.clone();
            app.wallet_profile_label = profile.label.clone();
            app.session.notice = Some(format!("Wallet profile loaded: {}", profile.label));
        }
        if secondary_button(ui, "Delete").clicked() {
            app.delete_selected_neo_wallet_profile();
        }
    });
}

fn format_optional_unix(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}
