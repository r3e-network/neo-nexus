use eframe::egui;

use crate::{
    app::domain::NodeType,
    app::{
        theme,
        widgets::{field_combo, field_text, form_group},
        NeoNexusApp,
    },
};

pub(super) fn render_package_fields(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    form_group(ui, "Package identity", |ui| {
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
    });
    ui.add_space(theme::MD);
    form_group(ui, "Platform & source", |ui| {
        render_platform_fields(app, ui);
        ui.add_space(theme::SM);
        field_text(ui, "Source", &mut app.runtime_package_draft.source_path);
        ui.add_space(theme::SM);
        field_text(
            ui,
            "Executable",
            &mut app.runtime_package_draft.executable_name,
        );
    });
    ui.add_space(theme::MD);
    form_group(ui, "Integrity", |ui| {
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
    });
}

pub(super) fn render_download_fields(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    form_group(ui, "HTTPS download", |ui| {
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
    });
}

fn render_platform_fields(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.columns(2, |columns| {
        field_text(&mut columns[0], "OS", &mut app.runtime_package_draft.os);
        field_text(&mut columns[1], "Arch", &mut app.runtime_package_draft.arch);
    });
}
