use eframe::egui;

use crate::app::domain::NeoWalletProfile;

use super::super::super::{
    paging::{page_count, rows_that_fit},
    text::truncate_middle,
    theme::{self, muted_text},
    widgets::{empty_state, pagination_bar},
    NeoNexusApp, WALLET_PROFILE_PAGE_SIZE,
};

use super::filter::render_wallet_profile_filter;

/// The selectable label/address button at the top of a registry row.
const WALLET_ROW_BUTTON_HEIGHT: f32 = 28.0;

/// A whole registry row: the button above, the secondary line carrying the
/// account counts and wallet hash, and the spacing egui puts around both.
const WALLET_ROW_HEIGHT: f32 = 64.0;

/// Chrome between the filter and the first row: the pagination bar and the
/// separator under it.
const WALLET_CHROME_HEIGHT: f32 = 52.0;

pub(super) fn render_wallet_profile_registry(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    render_wallet_profile_filter(app, ui);

    if app.neo_wallet_profiles.is_empty() {
        empty_state(
            ui,
            "No wallet profiles",
            "Import an encrypted NEP-6 wallet.",
        );
        return;
    }

    let profiles = app.filtered_neo_wallet_profiles();
    if profiles.is_empty() {
        empty_state(
            ui,
            "No matching profiles",
            "No wallet profiles match current filter.",
        );
        return;
    }

    // Measured after the filter, before the bar and separator are drawn, so
    // those two are subtracted as reserved space rather than read from here.
    let page_size = rows_that_fit(
        ui.available_height(),
        WALLET_ROW_HEIGHT,
        WALLET_CHROME_HEIGHT,
    )
    .min(WALLET_PROFILE_PAGE_SIZE);
    let total_pages = page_count(profiles.len(), page_size);
    app.wallet_profile_page = app.wallet_profile_page.min(total_pages - 1);
    pagination_bar(
        ui,
        &mut app.wallet_profile_page,
        total_pages,
        profiles.len(),
    );
    ui.separator();

    let start = app.wallet_profile_page * page_size;
    let visible = profiles
        .iter()
        .skip(start)
        .take(page_size)
        .cloned()
        .collect::<Vec<_>>();
    let mut next_selection = app.selected_neo_wallet_profile.clone();

    for row in 0..page_size {
        if let Some(profile) = visible.get(row) {
            if render_wallet_profile_row(
                ui,
                profile,
                next_selection.as_deref() == Some(profile.id.as_str()),
            ) {
                next_selection = Some(profile.id.clone());
            }
        } else {
            ui.add_space(WALLET_ROW_HEIGHT);
        }
    }

    if app.selected_neo_wallet_profile != next_selection {
        app.selected_neo_wallet_profile = next_selection;
    }
}

fn render_wallet_profile_row(
    ui: &mut egui::Ui,
    profile: &NeoWalletProfile,
    selected: bool,
) -> bool {
    let label = format!(
        "{}  {}",
        truncate_middle(&profile.label, 24),
        truncate_middle(&profile.primary_address, 18)
    );
    let response = ui.add_sized(
        [ui.available_width(), WALLET_ROW_BUTTON_HEIGHT],
        egui::Button::new(label).selected(selected),
    );
    ui.horizontal(|ui| {
        ui.add_space(theme::SM);
        ui.label(
            egui::RichText::new(format!(
                "{} encrypted / {} key(s)",
                profile.encrypted_account_count,
                profile.contract_public_keys.len()
            ))
            .color(muted_text()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(theme::SM);
            ui.label(truncate_middle(&profile.wallet_sha256, 18));
        });
    });
    response.clicked()
}
