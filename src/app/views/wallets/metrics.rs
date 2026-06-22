use eframe::egui;

use crate::app::domain::NeoWalletProfile;

use super::super::super::widgets::metric_row;

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

    let profiles_label = profile_count.to_string();
    let encrypted = encrypted_accounts.to_string();
    let default_label = default_accounts.to_string();
    let watch_only = watch_only_accounts.to_string();
    metric_row(
        ui,
        &[
            ("Profiles", &profiles_label, "stored locally"),
            ("Encrypted", &encrypted, "wallet accounts"),
            ("Default", &default_label, "primary accounts"),
            ("Watch-only", &watch_only, "observed accounts"),
        ],
    );
}
