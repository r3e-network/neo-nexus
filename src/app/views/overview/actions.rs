use eframe::egui;

use crate::app::domain::{
    evaluate_fleet, CheckSeverity, FleetDiagnostics, ReadinessAction,
};

use super::super::super::{
    text::truncate_middle,
    theme,
    view::View,
    widgets::{
        empty_state, primary_button, secondary_button, severity_badge, text_badge,
    },
    NeoNexusApp, views::OperationsSection,
};

const HOME_ACTION_LIMIT: usize = 5;

pub(super) fn render_next_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let plugin_states = app.plugin_states_by_node();
    let diagnostics = evaluate_fleet(&app.nodes, &plugin_states);
    let mut actions = app.filtered_readiness_actions(&diagnostics);
    // Surface the most severe items first so Home is a triage desk, not a log.
    actions.sort_by(|left, right| {
        severity_rank(right.severity)
            .cmp(&severity_rank(left.severity))
            .then_with(|| left.node_name.cmp(&right.node_name))
    });
    let total = actions.len();
    let top: Vec<ReadinessAction> = actions.into_iter().take(HOME_ACTION_LIMIT).collect();

    if top.is_empty() {
        empty_state(
            ui,
            "All clear",
            "No open readiness actions. Fleet checks are clean.",
        );
        return;
    }

    ui.horizontal(|ui| {
        ui.label(theme::muted_body(format!(
            "Showing {} of {total} open actions",
            top.len()
        )));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if secondary_button(ui, "Open Operations").clicked() {
                app.selected_view = View::Operations;
                app.operations_section = OperationsSection::ActionQueue;
            }
        });
    });
    ui.add_space(theme::SM);

    for action in &top {
        render_action_card(app, ui, action, &diagnostics);
        ui.add_space(theme::XS);
    }
}

fn render_action_card(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    action: &ReadinessAction,
    _diagnostics: &FleetDiagnostics,
) {
    let width = ui.available_width();
    egui::Frame::new()
        .fill(theme::card_surface())
        .stroke(theme::hairline())
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_min_width(width - 4.0);
            ui.horizontal(|ui| {
                severity_badge(ui, action.severity);
                ui.add_space(theme::SM);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(theme::body(truncate_middle(&action.node_name, 18)).strong());
                        ui.add_space(theme::XS);
                        text_badge(ui, &format!("score {}", action.node_score), theme::muted_text());
                    });
                    ui.add_space(2.0);
                    ui.label(theme::body(truncate_middle(&action.title, 42)));
                    ui.label(theme::muted_body(truncate_middle(&action.detail, 64)));
                });
            });
            ui.add_space(theme::SM);
            ui.horizontal(|ui| {
                if primary_button(ui, action.resolution.action_label()).clicked() {
                    app.open_readiness_action_resolution(action);
                }
                if secondary_button(ui, "Select node").clicked() {
                    app.select_fleet_node(Some(action.node_id.clone()));
                }
            });
        });
}

fn severity_rank(severity: CheckSeverity) -> u8 {
    match severity {
        CheckSeverity::Critical => 3,
        CheckSeverity::Warning => 2,
        CheckSeverity::Info => 1,
        CheckSeverity::Pass => 0,
    }
}
