use eframe::egui;

use crate::{
    app::domain::NodeType,
    app::widgets::{labeled_combo, labeled_text},
    app::NeoNexusApp,
};

pub(super) fn render_package_fields(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    labeled_text(ui, "ID", &mut app.runtime_package_draft.id);
    labeled_text(ui, "Label", &mut app.runtime_package_draft.label);
    labeled_combo(
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
    labeled_text(ui, "Version", &mut app.runtime_package_draft.version);
    render_platform_fields(app, ui);
    labeled_text(ui, "Source", &mut app.runtime_package_draft.source_path);
    labeled_text(
        ui,
        "Executable",
        &mut app.runtime_package_draft.executable_name,
    );
    labeled_text(
        ui,
        "SHA-256",
        &mut app.runtime_package_draft.expected_sha256,
    );
    labeled_text(
        ui,
        "Signature",
        &mut app.runtime_package_draft.signature_path,
    );
    labeled_text(
        ui,
        "Public key",
        &mut app.runtime_package_draft.ed25519_public_key,
    );
}

pub(super) fn render_download_fields(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    labeled_text(ui, "HTTPS URL", &mut app.runtime_package_draft.download_url);
    labeled_text(
        ui,
        "File name",
        &mut app.runtime_package_draft.download_file_name,
    );
    ui.horizontal(|ui| {
        ui.label("Max");
        ui.add(
            egui::DragValue::new(&mut app.runtime_package_draft.download_max_mib)
                .range(1..=4096)
                .suffix(" MiB")
                .speed(16.0),
        );
    });
}

fn render_platform_fields(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Platform");
        ui.add_sized(
            [ui.available_width() * 0.5, 24.0],
            egui::TextEdit::singleline(&mut app.runtime_package_draft.os),
        );
        ui.add_sized(
            [ui.available_width().max(100.0), 24.0],
            egui::TextEdit::singleline(&mut app.runtime_package_draft.arch),
        );
    });
}
