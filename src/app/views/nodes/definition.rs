use eframe::egui;

use crate::app::domain::{Network, NodeType, StorageEngine};

use super::super::super::{
    theme,
    widgets::{
        callout, field_combo, field_text, form_group, toolbar, CalloutKind, ToolbarAction,
    },
    NeoNexusApp,
};

pub(super) fn render_create_form(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    app.fleet.draft.ensure_storage_compatible();

    ui.label(theme::label_caption("Draft definition"));
    ui.add_space(theme::SM);

    ui.columns(2, |columns| {
        form_group(&mut columns[0], "Identity", |ui| {
            render_identity_fields(app, ui);
        });
        form_group(&mut columns[1], "Executable & ports", |ui| {
            render_executable_fields(app, ui);
        });
    });

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

fn render_identity_fields(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    field_text(ui, "Name", &mut app.fleet.draft.name);
    ui.add_space(theme::SM);
    let previous_node_type = app.fleet.draft.node_type;
    field_combo(
        ui,
        "Type",
        "node_type",
        app.fleet.draft.node_type.to_string(),
        |ui| {
            for node_type in NodeType::ALL {
                ui.selectable_value(&mut app.fleet.draft.node_type, node_type, node_type.to_string());
            }
        },
    );
    if previous_node_type != app.fleet.draft.node_type {
        app.fleet.draft.ensure_storage_compatible();
    }
    ui.add_space(theme::SM);
    field_combo(
        ui,
        "Network",
        "network",
        app.fleet.draft.network.to_string(),
        |ui| {
            for network in Network::ALL {
                ui.selectable_value(&mut app.fleet.draft.network, network, network.to_string());
            }
        },
    );
    ui.add_space(theme::SM);
    field_combo(
        ui,
        "Storage",
        "storage",
        app.fleet.draft.storage_engine.to_string(),
        |ui| {
            for engine in StorageEngine::ALL {
                if app.fleet.draft.node_type.supports_storage_engine(engine) {
                    ui.selectable_value(&mut app.fleet.draft.storage_engine, engine, engine.to_string());
                }
            }
        },
    );
}

fn render_executable_fields(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    field_text(ui, "Binary", &mut app.fleet.draft.binary_path);
    ui.add_space(theme::SM);
    field_text(ui, "Version", &mut app.fleet.draft.runtime_version);
    ui.add_space(theme::SM);
    field_text(ui, "Args", &mut app.fleet.draft.args);
    ui.add_space(theme::MD);
    ui.label(theme::label_caption("Network ports"));
    ui.add_space(theme::XS);
    field_text(ui, "RPC", &mut app.fleet.draft.rpc_port);
    ui.add_space(theme::SM);
    field_text(ui, "P2P", &mut app.fleet.draft.p2p_port);
    ui.add_space(theme::SM);
    field_text(ui, "WebSocket", &mut app.fleet.draft.ws_port);
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
