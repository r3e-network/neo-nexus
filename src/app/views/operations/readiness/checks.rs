use eframe::egui;

use crate::app::widgets::inset_card;

use crate::app::domain::{DiagnosticCheck, NodeDiagnostics};

use super::super::super::super::widgets::empty_state;
use super::super::{
    super::super::{
        paging::{page_count, rows_that_fit},
        text::truncate_middle,
        theme,
        widgets::{list_row_frame, pagination_bar, primary_button, severity_badge, text_badge},
        NeoNexusApp, READINESS_CHECK_PAGE_SIZE,
    },
    helpers::score_color,
};

/// Pitch of one check row plus the `XS` gap after it. The row is content-height
/// (title over a wrapped detail line), so it is measured rather than declared:
/// at the 1280x820 design window it lands on 83pt in the narrow column the open
/// inspector leaves, and 68pt when the inspector is hidden. Paging on the taller
/// reading keeps every row inside the panel in both layouts.
const CHECK_ROW_PITCH: f32 = 83.0;

/// Chrome this function draws around the rows: the pagination bar with its
/// trailing `SM` gap above (47pt), and the selected-check card below (167pt).
/// The card is unconditional — `ensure_visible_readiness_check_selection` always
/// leaves a check selected for a non-empty list.
const CHECK_CHROME: f32 = 47.0 + 167.0;

pub(super) fn render_checks(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    node: &NodeDiagnostics,
    checks: &[DiagnosticCheck],
) {
    if checks.is_empty() {
        empty_state(ui, "No matching checks", "Adjust the readiness filter.");
        return;
    }

    app.ensure_visible_readiness_check_selection(checks);
    // Read before the pagination bar goes in, so the bar is part of the reserve
    // rather than something already subtracted from the measurement.
    let page_size = rows_that_fit(ui.available_height(), CHECK_ROW_PITCH, CHECK_CHROME)
        .min(READINESS_CHECK_PAGE_SIZE);
    let total_pages = page_count(checks.len(), page_size);
    app.operations_ui.readiness_check_page =
        app.operations_ui.readiness_check_page.min(total_pages - 1);
    pagination_bar(
        ui,
        &mut app.operations_ui.readiness_check_page,
        total_pages,
        checks.len(),
    );
    ui.add_space(theme::SM);

    let start = app.operations_ui.readiness_check_page * page_size;
    for check in checks.iter().skip(start).take(page_size) {
        render_check_row(app, ui, check);
        ui.add_space(theme::XS);
    }
    render_selected_check_summary(app, ui, node, checks);
}

fn render_check_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, check: &DiagnosticCheck) {
    let selected = app
        .operations_ui
        .selected_readiness_check
        .as_ref()
        .is_some_and(|key| key.matches(check));
    // Content-height list row — no fixed min height (v3.1 readiness contract).
    if list_row_frame(ui, selected, None, |ui| {
        ui.horizontal(|ui| {
            severity_badge(ui, check.severity);
            ui.add_space(theme::SM);
            ui.vertical(|ui| {
                ui.label(theme::body(check.title).strong());
                ui.label(theme::muted_body(truncate_middle(&check.detail, 72)));
            });
        });
    }) {
        app.select_readiness_check(check);
    }
}

fn render_selected_check_summary(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    node: &NodeDiagnostics,
    checks: &[DiagnosticCheck],
) {
    let Some(check) = app.selected_visible_readiness_check(checks).cloned() else {
        return;
    };

    ui.add_space(theme::SM);
    inset_card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(theme::label_caption("Selected check"));
            ui.add_space(theme::SM);
            severity_badge(ui, check.severity);
            ui.add_space(theme::XS);
            text_badge(ui, check.resolution.label(), theme::muted_text());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    theme::body(format!("{}% ready", node.score))
                        .color(score_color(node.score))
                        .strong(),
                );
            });
        });
        ui.add_space(theme::SM);
        ui.label(theme::body(check.title).strong());
        ui.label(theme::muted_body(check.detail.as_str()));
        ui.add_space(theme::SM);
        if primary_button(ui, check.resolution.action_label())
            .on_hover_text(check.resolution.hint())
            .clicked()
        {
            app.open_readiness_check_resolution(node, &check);
        }
    });
}
