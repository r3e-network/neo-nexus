use eframe::egui;

use crate::app::{
    domain::NodeType,
    theme,
    widgets::{columns_that_fit, field_combo, field_text, form_group},
    NeoNexusApp,
};

/// Narrowest a labelled field column may become before its eyebrow label wraps.
pub(super) const FIELD_COLUMN_MIN_WIDTH: f32 = 176.0;

/// The groups that describe the package being installed. Flowing them across
/// columns rather than stacking them is what keeps the install form inside a
/// non-scrolling panel: eleven fields in one column do not fit.
type FieldGroup = (&'static str, fn(&mut NeoNexusApp, &mut egui::Ui));
const PACKAGE_GROUPS: [FieldGroup; 4] = [
    ("Package identity", identity),
    ("Platform & source", platform),
    ("Integrity", integrity),
    ("HTTPS download", download),
];

pub(super) fn render_package_fields(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let per_row = columns_that_fit(
        ui.available_width(),
        FIELD_COLUMN_MIN_WIDTH,
        PACKAGE_GROUPS.len(),
    );
    for (row_index, row) in PACKAGE_GROUPS.chunks(per_row).enumerate() {
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

fn identity(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    field_text(ui, "ID", &mut app.runtime_package_draft.id);
    ui.add_space(theme::SM);
    field_text(ui, "Label", &mut app.runtime_package_draft.label);
    ui.add_space(theme::SM);
    field_combo(
        ui,
        "Runtime",
        "runtime_package_node_type",
        app.runtime_package_draft.node_type.to_string(),
        |ui| {
            for node_type in NodeType::ALL {
                ui.selectable_value(
                    &mut app.runtime_package_draft.node_type,
                    node_type,
                    node_type.to_string(),
                );
            }
        },
    );
    ui.add_space(theme::SM);
    field_text(ui, "Version", &mut app.runtime_package_draft.version);
}

fn platform(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    // OS and Arch are stacked, not paired into two sub-columns. This group is
    // itself already one column of a reflowing row, and `field_text` floors its
    // editor at 120pt — half of a 240pt group is narrower than that, so the
    // pair would push the whole form past the edge of the workspace.
    field_text(ui, "OS", &mut app.runtime_package_draft.os);
    ui.add_space(theme::SM);
    field_text(ui, "Arch", &mut app.runtime_package_draft.arch);
    ui.add_space(theme::SM);
    field_text(ui, "Source", &mut app.runtime_package_draft.source_path);
    ui.add_space(theme::SM);
    field_text(
        ui,
        "Executable",
        &mut app.runtime_package_draft.executable_name,
    );
}

fn integrity(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    field_text(
        ui,
        "SHA-256",
        &mut app.runtime_package_draft.expected_sha256,
    );
    ui.add_space(theme::SM);
    field_text(
        ui,
        "Signature",
        &mut app.runtime_package_draft.signature_path,
    );
    ui.add_space(theme::SM);
    field_text(
        ui,
        "Public key",
        &mut app.runtime_package_draft.ed25519_public_key,
    );
}

fn download(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    field_text(ui, "URL", &mut app.runtime_package_draft.download_url);
    ui.add_space(theme::SM);
    field_text(
        ui,
        "File name",
        &mut app.runtime_package_draft.download_file_name,
    );
    ui.add_space(theme::SM);
    ui.label(theme::label_caption("Max size"));
    ui.add_space(theme::XS);
    ui.add(
        egui::DragValue::new(&mut app.runtime_package_draft.download_max_mib)
            .range(1..=4096)
            .suffix(" MiB")
            .speed(16.0),
    );
}
