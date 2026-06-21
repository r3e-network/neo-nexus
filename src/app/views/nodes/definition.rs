use eframe::egui;

use crate::app::domain::{Network, NodeType, StorageEngine};

use super::super::super::{
    theme::muted_text,
    widgets::{labeled_combo, labeled_text},
    NeoNexusApp,
};

pub(super) fn render_create_form(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    app.draft.ensure_storage_compatible();
    ui.label(egui::RichText::new("Draft").color(muted_text()));
    ui.separator();

    ui.columns(2, |columns| {
        render_identity_column(app, &mut columns[0]);
        render_executable_column(app, &mut columns[1]);
    });

    ui.add_space(10.0);
    action_bar(app, ui);
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("Running nodes must be stopped before definition changes.")
            .color(muted_text()),
    );
}

fn render_identity_column(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.strong("Identity");
    labeled_text(ui, "Name", &mut app.draft.name);
    let previous_node_type = app.draft.node_type;
    labeled_combo(
        ui,
        "Type",
        "node_type",
        app.draft.node_type.to_string(),
        |ui| {
            for node_type in NodeType::ALL {
                ui.selectable_value(&mut app.draft.node_type, node_type, node_type.to_string());
            }
        },
    );
    if previous_node_type != app.draft.node_type {
        app.draft.ensure_storage_compatible();
    }
    labeled_combo(
        ui,
        "Network",
        "network",
        app.draft.network.to_string(),
        |ui| {
            for network in Network::ALL {
                ui.selectable_value(&mut app.draft.network, network, network.to_string());
            }
        },
    );
    labeled_combo(
        ui,
        "Storage",
        "storage",
        app.draft.storage_engine.to_string(),
        |ui| {
            for engine in StorageEngine::ALL {
                if app.draft.node_type.supports_storage_engine(engine) {
                    ui.selectable_value(&mut app.draft.storage_engine, engine, engine.to_string());
                }
            }
        },
    );
}

fn render_executable_column(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.strong("Executable");
    labeled_text(ui, "Binary", &mut app.draft.binary_path);
    labeled_text(ui, "Version", &mut app.draft.runtime_version);
    labeled_text(ui, "Args", &mut app.draft.args);
    ui.separator();
    ui.strong("Network ports");
    labeled_text(ui, "RPC", &mut app.draft.rpc_port);
    labeled_text(ui, "P2P", &mut app.draft.p2p_port);
    labeled_text(ui, "WS", &mut app.draft.ws_port);
}

fn action_bar(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui
            .add_sized([112.0, 30.0], egui::Button::new("Save New"))
            .clicked()
        {
            app.create_node();
        }
        if ui.button("Probe Draft").clicked() {
            app.probe_draft_binary();
        }
        if ui.button("Smoke Draft").clicked() {
            app.smoke_draft_runtime();
        }
        if ui.button("Auto Ports").clicked() {
            app.auto_assign_draft_ports();
        }

        let selected_can_edit = app
            .selected_node()
            .is_some_and(|node| !node.status.is_running());
        if ui
            .add_enabled(selected_can_edit, egui::Button::new("Update Selected"))
            .clicked()
        {
            app.update_selected_node();
        }

        if ui.button("Reset Draft").clicked() {
            app.draft = Default::default();
            app.notice = Some("Draft reset".to_string());
        }
    });
}
