use eframe::egui;

use crate::app::domain::{DiagnosticCheck, NodeDiagnostics};

use super::super::super::super::widgets::empty_state;
use super::super::{
    super::super::{
        paging::page_count,
        text::truncate_middle,
        theme,
        widgets::{pagination_bar, primary_button, severity_badge, text_badge},
        NeoNexusApp, READINESS_CHECK_PAGE_SIZE,
    },
    helpers::score_color,
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
    ui.add_space(theme::SM);

    let start = app.readiness_check_page * READINESS_CHECK_PAGE_SIZE;
    for check in checks.iter().skip(start).take(READINESS_CHECK_PAGE_SIZE) {
        render_check_row(app, ui, check);
        ui.add_space(theme::XS);
    }
    render_selected_check_summary(app, ui, node, checks);
}

fn render_check_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, check: &DiagnosticCheck) {
    let selected = app
        .selected_readiness_check
        .as_ref()
        .is_some_and(|key| key.matches(check));
    let fill = if selected {
        theme::accent().gamma_multiply(0.14)
    } else {
        theme::card_surface()
    };
    let stroke = if selected {
        egui::Stroke::new(1.0, theme::accent())
    } else {
        theme::hairline()
    };
    let response = egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                severity_badge(ui, check.severity);
                ui.add_space(theme::SM);
                ui.vertical(|ui| {
                    ui.label(theme::body(check.title).strong());
                    ui.label(theme::muted_body(truncate_middle(&check.detail, 72)));
                });
            });
        })
        .response
        .interact(egui::Sense::click());
    if response.clicked() {
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
    egui::Frame::new()
        .fill(theme::card_surface())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
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
