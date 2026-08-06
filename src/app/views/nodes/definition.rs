use eframe::egui;

use super::super::super::{
    theme,
    widgets::{callout, columns_that_fit, form_group, toolbar, CalloutKind, ToolbarAction},
    NeoNexusApp,
};

mod fields;

/// Narrowest a labelled form field column may become. Below this the eyebrow
/// label wraps and the combo boxes clip their own selected text.
const FIELD_COLUMN_MIN_WIDTH: f32 = 176.0;

/// The definition's field groups, in the order an operator fills them.
type FieldGroup = (&'static str, fn(&mut NeoNexusApp, &mut egui::Ui));
const GROUPS: [FieldGroup; 3] = [
    ("Identity", fields::identity),
    ("Runtime", fields::runtime),
    ("Ports", fields::ports),
];

pub(super) fn render_create_form(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    app.fleet.draft.ensure_storage_compatible();

    ui.label(theme::label_caption("Draft definition"));
    ui.add_space(theme::SM);
    render_field_groups(app, ui);

    ui.add_space(theme::MD);
    action_bar(app, ui);
    ui.add_space(theme::SM);
    callout(
        ui,
        CalloutKind::Warning,
        "Running nodes are locked",
        "Stop a node before updating its definition, ports, or binary path.",
    );
}

/// Lays the three groups across as many columns as the pane can hold at
/// [`FIELD_COLUMN_MIN_WIDTH`], wrapping onto further rows below that.
///
/// The column count matters for more than looks: the workspace does not
/// scroll, so one stacked column of all eleven fields paints its last rows
/// below the panel edge, where they cannot be read or clicked.
fn render_field_groups(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let per_row = columns_that_fit(ui.available_width(), FIELD_COLUMN_MIN_WIDTH, GROUPS.len());
    for (row_index, row) in GROUPS.chunks(per_row).enumerate() {
        if row_index > 0 {
            ui.add_space(theme::MD);
        }
        if per_row == 1 {
            for (title, render) in row {
                form_group(ui, title, |ui| render(app, ui));
            }
            continue;
        }
        ui.columns(per_row, |columns| {
            for (column, (title, render)) in columns.iter_mut().zip(row) {
                form_group(column, title, |ui| render(app, ui));
            }
        });
    }
}

fn action_bar(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let selected_can_edit = app
        .selected_node()
        .is_some_and(|node| !node.status.is_running());
    let actions = [
        ToolbarAction::primary("save", "Save New").hint("Create a node from this draft"),
        ToolbarAction::secondary("probe", "Probe Draft").hint("Inspect the draft binary path"),
        ToolbarAction::secondary("smoke", "Smoke Draft").hint("Run a short runtime smoke probe"),
        ToolbarAction::secondary("ports", "Auto Ports").hint("Assign free RPC/P2P/WS ports"),
        ToolbarAction::secondary("update", "Update Selected")
            .enabled(selected_can_edit)
            .hint("Write the draft onto the selected stopped node"),
        ToolbarAction::secondary("reset", "Reset Draft").hint("Clear the draft form"),
    ];
    if let Some(id) = toolbar(ui, &actions) {
        match id {
            "save" => app.create_node(),
            "probe" => app.probe_draft_binary(),
            "smoke" => app.smoke_draft_runtime(),
            "ports" => app.auto_assign_draft_ports(),
            "update" => app.update_selected_node(),
            "reset" => {
                app.fleet.draft = Default::default();
                app.session.notice = Some("Draft reset".to_string());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/nodes/definition/tests.rs"]
mod tests;
