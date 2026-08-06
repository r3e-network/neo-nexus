use eframe::egui;

use crate::app::domain::{evaluate_fleet, CheckSeverity, FleetDiagnostics, ReadinessAction};

use super::super::super::{
    paging::rows_that_fit,
    text::truncate_middle,
    theme,
    view::View,
    views::OperationsSection,
    widgets::{
        empty_state, inset_card, primary_button, secondary_button, severity_badge, text_badge,
    },
    NeoNexusApp,
};

/// Height of one action card: severity badge row, title, detail line, and the
/// resolve/select button row, plus the inset card's own margins and the gap to
/// the next card.
const ACTION_CARD_HEIGHT: f32 = 132.0;

/// Space the "Showing N of M" header and its Open Operations button take above
/// the first card.
const ACTIONS_HEADER_HEIGHT: f32 = 44.0;

pub(super) fn render_next_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let plugin_states = app.plugin_states_by_node();
    let diagnostics = evaluate_fleet(&app.fleet.nodes, &plugin_states);
    let mut actions = app.filtered_readiness_actions(&diagnostics);
    // Surface the most severe items first so Home is a triage desk, not a log.
    actions.sort_by(|left, right| {
        severity_rank(right.severity)
            .cmp(&severity_rank(left.severity))
            .then_with(|| left.node_name.cmp(&right.node_name))
    });
    let total = actions.len();
    // Derived from the room this panel actually has, not a fixed count: the
    // workbench does not scroll, so surplus cards would be painted below the
    // panel edge where they cannot be seen or clicked.
    let limit = rows_that_fit(
        ui.available_height(),
        ACTION_CARD_HEIGHT,
        ACTIONS_HEADER_HEIGHT,
    );
    let top: Vec<ReadinessAction> = actions.into_iter().take(limit).collect();

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
                app.session.selected_view = View::Operations;
                app.operations_ui.section = OperationsSection::ActionQueue;
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
    inset_card(ui, |ui| {
        ui.horizontal(|ui| {
            severity_badge(ui, action.severity);
            ui.add_space(theme::SM);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(theme::body(truncate_middle(&action.node_name, 18)).strong());
                    ui.add_space(theme::XS);
                    text_badge(
                        ui,
                        &format!("score {}", action.node_score),
                        theme::muted_text(),
                    );
                });
                ui.add_space(theme::XS);
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
