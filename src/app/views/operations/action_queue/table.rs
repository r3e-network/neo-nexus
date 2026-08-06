use eframe::egui;

use crate::app::widgets::inset_card;

use crate::app::domain::ReadinessAction;

use super::super::{
    super::super::{
        text::truncate_middle,
        theme,
        widgets::{grid_header, primary_button, secondary_button, severity_badge, text_badge},
        NeoNexusApp,
    },
    helpers::score_color,
};

pub(super) fn render_action_table(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    actions: &[ReadinessAction],
    start: usize,
    page_size: usize,
) {
    egui::Grid::new("operations_action_queue")
        .striped(true)
        .min_col_width(62.0)
        .show(ui, |ui| {
            grid_header(ui, &["Severity", "Node", "Score", "Check", "Detail"]);

            for action in actions.iter().skip(start).take(page_size) {
                render_action_row(app, ui, action);
                ui.end_row();
            }
        });
}

pub(super) fn render_selected_action_summary(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    actions: &[ReadinessAction],
) {
    let Some(action) = app.selected_visible_readiness_action(actions).cloned() else {
        return;
    };

    ui.add_space(theme::SM);
    inset_card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(theme::label_caption("Selected action"));
            ui.add_space(theme::SM);
            severity_badge(ui, action.severity);
            ui.add_space(theme::XS);
            text_badge(ui, action.resolution.label(), theme::muted_text());
        });
        ui.add_space(theme::SM);
        ui.label(theme::body(format!("{} — {}", action.node_name, action.title)).strong());
        ui.label(theme::muted_body(&action.detail));
        ui.add_space(theme::SM);
        ui.horizontal(|ui| {
            if primary_button(ui, action.resolution.action_label())
                .on_hover_text(action.resolution.hint())
                .clicked()
            {
                app.open_readiness_action_resolution(&action);
            }
            if secondary_button(ui, "Select node").clicked() {
                app.select_fleet_node(Some(action.node_id.clone()));
            }
        });
    });
}

fn render_action_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, action: &ReadinessAction) {
    severity_badge(ui, action.severity);
    let selected = app
        .operations_ui
        .selected_readiness_action
        .as_ref()
        .is_some_and(|key| key.matches(action));
    if ui
        .selectable_label(selected, truncate_middle(&action.node_name, 18))
        .on_hover_text(action.node_name.as_str())
        .clicked()
    {
        app.select_readiness_action(action);
    }
    ui.label(
        theme::body(format!("{}%", action.node_score))
            .color(score_color(action.node_score))
            .strong(),
    );
    ui.label(truncate_middle(&action.title, 18))
        .on_hover_text(action.title.as_str());
    ui.label(truncate_middle(&action.detail, 44))
        .on_hover_text(action.detail.as_str());
}
