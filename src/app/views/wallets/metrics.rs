use eframe::egui;

use crate::wallet::NeoWalletProfile;

use super::super::super::widgets::metric_tile;

pub(super) fn render_wallet_metrics(ui: &mut egui::Ui, profiles: &[NeoWalletProfile]) {
    let profile_count = profiles.len();
    let encrypted_accounts = profiles
        .iter()
        .map(|profile| profile.encrypted_account_count)
        .sum::<usize>();
    let default_accounts = profiles
        .iter()
        .map(|profile| profile.default_account_count)
        .sum::<usize>();
    let watch_only_accounts = profiles
        .iter()
        .map(|profile| profile.watch_only_account_count)
        .sum::<usize>();

    ui.horizontal(|ui| {
        metric_tile(ui, "Profiles", &profile_count.to_string(), "stored locally");
        metric_tile(
            ui,
            "Encrypted",
            &encrypted_accounts.to_string(),
            "wallet accounts",
        );
        metric_tile(
            ui,
            "Default",
            &default_accounts.to_string(),
            "primary accounts",
        );
        metric_tile(
            ui,
            "Watch-only",
            &watch_only_accounts.to_string(),
            "observed accounts",
        );
    });
}
