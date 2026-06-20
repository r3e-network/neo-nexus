use eframe::egui;

use crate::diagnostics::{DiagnosticCheck, NodeDiagnostics};

use super::super::super::super::widgets::empty_state;
use super::super::{
    super::super::{
        paging::page_count, text::truncate_middle, theme::muted_text, widgets::pagination_bar,
        NeoNexusApp, READINESS_CHECK_PAGE_SIZE,
    },
    helpers::severity_color,
};

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
    let total_pages = page_count(checks.len(), READINESS_CHECK_PAGE_SIZE);
    app.readiness_check_page = app.readiness_check_page.min(total_pages - 1);
    pagination_bar(ui, &mut app.readiness_check_page, total_pages, checks.len());
    ui.separator();

    let start = app.readiness_check_page * READINESS_CHECK_PAGE_SIZE;
    for check in checks.iter().skip(start).take(READINESS_CHECK_PAGE_SIZE) {
        render_check_row(app, ui, check);
    }
    render_selected_check_summary(app, ui, node, checks);
}

fn render_check_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, check: &DiagnosticCheck) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(check.severity.label())
                .strong()
                .color(severity_color(check.severity)),
        );
        let selected = app
            .selected_readiness_check
            .as_ref()
            .is_some_and(|key| key.matches(check));
        if ui
            .selectable_label(selected, check.title)
            .on_hover_text(check.title)
            .clicked()
        {
            app.select_readiness_check(check);
        }
        ui.label(truncate_middle(&check.detail, 54))
            .on_hover_text(check.detail.as_str());
    });
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

    ui.separator();
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Selected").color(muted_text()));
        ui.label(
            egui::RichText::new(check.severity.label())
                .strong()
                .color(severity_color(check.severity)),
        );
        ui.label(check.title).on_hover_text(check.title);
        ui.label(truncate_middle(&check.detail, 52))
            .on_hover_text(check.detail.as_str());
        ui.label(
            egui::RichText::new(check.resolution.label())
                .strong()
                .color(muted_text()),
        );
        if ui
            .button(check.resolution.action_label())
            .on_hover_text(check.resolution.hint())
            .clicked()
        {
            app.open_readiness_check_resolution(node, &check);
        }
    });
}
