use eframe::egui;

use crate::diagnostics::ReadinessAction;

use super::super::{
    super::super::{text::truncate_middle, theme::muted_text, NeoNexusApp, ACTION_QUEUE_PAGE_SIZE},
    helpers::{score_color, severity_color},
};

pub(super) fn render_action_table(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    actions: &[ReadinessAction],
    start: usize,
) {
    egui::Grid::new("operations_action_queue")
        .striped(true)
        .min_col_width(62.0)
        .show(ui, |ui| {
            ui.strong("Severity");
            ui.strong("Node");
            ui.strong("Score");
            ui.strong("Check");
            ui.strong("Detail");
            ui.end_row();

            for action in actions.iter().skip(start).take(ACTION_QUEUE_PAGE_SIZE) {
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

    ui.separator();
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Selected").color(muted_text()));
        ui.label(
            egui::RichText::new(action.severity.label())
                .strong()
                .color(severity_color(action.severity)),
        );
        ui.label(truncate_middle(&action.node_name, 20))
            .on_hover_text(action.node_name.as_str());
        ui.label(truncate_middle(&action.title, 22))
            .on_hover_text(action.title.as_str());
        ui.label(truncate_middle(&action.detail, 52))
            .on_hover_text(action.detail.as_str());
        ui.label(
            egui::RichText::new(action.resolution.label())
                .strong()
                .color(muted_text()),
        );
        if ui
            .button(action.resolution.action_label())
            .on_hover_text(action.resolution.hint())
            .clicked()
        {
            app.open_readiness_action_resolution(&action);
        }
    });
}

fn render_action_row(app: &mut NeoNexusApp, ui: &mut egui::Ui, action: &ReadinessAction) {
    ui.label(
        egui::RichText::new(action.severity.label())
            .strong()
            .color(severity_color(action.severity)),
    );
    let selected = app
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
        egui::RichText::new(format!("{}%", action.node_score))
            .color(score_color(action.node_score)),
    );
    ui.label(truncate_middle(&action.title, 18))
        .on_hover_text(action.title.as_str());
    ui.label(truncate_middle(&action.detail, 44))
        .on_hover_text(action.detail.as_str());
}
