use eframe::egui;

use crate::app::{
    domain::RemoteServerProfile,
    paging::{page_count, rows_that_fit},
    text::truncate_middle,
    theme::DensityMetrics,
    widgets::{empty_state, pagination_bar},
    NeoNexusApp, REMOTE_SERVER_PAGE_SIZE,
};

use super::filter::render_remote_profile_filter;

/// A profile row is a bespoke two-line button: the profile name over its base
/// URL and enabled flag.
const PROFILE_ROW_HEIGHT: f32 = 54.0;

/// Chrome below the rows: the closing separator and the pagination bar.
const PROFILE_CHROME_HEIGHT: f32 = 52.0;

pub(super) fn render_remote_profile_list(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    render_remote_profile_filter(app, ui);

    if app.remote_servers.is_empty() {
        empty_state(
            ui,
            "No remotes",
            "Add a remote NeoNexus public endpoint profile.",
        );
        return;
    }

    let profiles = app.filtered_remote_server_profiles();
    if profiles.is_empty() {
        empty_state(
            ui,
            "No matching remotes",
            "No remote profiles match current filter.",
        );
        return;
    }

    // Measured here, after the filter, because that is where the rows begin;
    // the separator and pagination bar under them come off as reserved space.
    let row_height =
        PROFILE_ROW_HEIGHT + DensityMetrics::for_density(app.session.density).item_spacing_y;
    let page_size = rows_that_fit(ui.available_height(), row_height, PROFILE_CHROME_HEIGHT)
        .min(REMOTE_SERVER_PAGE_SIZE);
    let total_pages = page_count(profiles.len(), page_size);
    app.remote_server_page = app.remote_server_page.min(total_pages - 1);
    let start = app.remote_server_page * page_size;
    let page_profiles = profiles
        .iter()
        .skip(start)
        .take(page_size)
        .cloned()
        .collect::<Vec<_>>();
    let page_len = page_profiles.len();

    for profile in page_profiles {
        if render_remote_profile_row(ui, &profile, app.selected_remote_server.as_deref()) {
            app.selected_remote_server = Some(profile.id.clone());
            app.remote_probe_history_page = 0;
        }
    }

    for _ in page_len..page_size {
        ui.label(" ");
    }

    ui.separator();
    pagination_bar(ui, &mut app.remote_server_page, total_pages, profiles.len());
}

fn render_remote_profile_row(
    ui: &mut egui::Ui,
    profile: &RemoteServerProfile,
    selected_id: Option<&str>,
) -> bool {
    let selected = selected_id == Some(profile.id.as_str());
    let label = format!(
        "{}\n{}    {}",
        truncate_middle(&profile.name, 28),
        truncate_middle(&profile.base_url, 34),
        if profile.enabled { "on" } else { "off" }
    );
    ui.add_sized(
        [ui.available_width(), PROFILE_ROW_HEIGHT],
        egui::Button::new(label).selected(selected),
    )
    .on_hover_text(&profile.base_url)
    .clicked()
}
